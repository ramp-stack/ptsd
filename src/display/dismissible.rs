use prism::Context;
use prism::drawable::{Area, Component, Drawable, Layout, SizeRange};
use prism::events::{
    on_change, OnChange, DynEvent, MouseButton, MouseEvent, MouseState, Timer, WindowEvent,
};

use std::time::{Duration, Instant};

/// The edge a panel slides toward when dismissed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismissEdge {Top, Bottom}

/// A panel that follows a drag toward its edge, then dismisses or returns to rest.
///
/// Its owner calls `observe` before routing events to children. Drags begin in the 12-pixel
/// content-facing edge strip or an additional handle area supplied by the owner.
/// Movement up to 8 logical pixels leaves the panel at rest; captured drags cancel child input.
/// Its owner runs the winning callback on this panel, then calls `advance`.
#[derive(Debug, Component, Clone)]
pub struct Dismissible<D: Drawable + Clone> {
    layout: SlideLayout,
    pub child: D,
    #[skip] drag: Drag,
    #[skip] timer: Timer,
    #[skip] momentum: Option<(f32, f32)>,
    #[skip] area: Option<Area>,
}

impl<D: Drawable + Clone> Dismissible<D> {
    pub fn new(child: D, edge: DismissEdge) -> Self {
        Self {
            layout: SlideLayout {
                edge, size: (0.0, 0.0), child: SizeRange::default(), offset: 0.0,
                reserve: false, drag_height: None,
            },
            child, drag: Drag::Idle, timer: Timer::default(), momentum: None, area: None,
        }
    }

    /// Full panel size from its latest layout, before subtracting the dragged distance.
    pub fn size(&self) -> (f32, f32) {self.layout.size}

    /// Keep the occupied space while moving, for overlays or full-screen panels.
    /// By default the occupied height shrinks, allowing adjacent content to expand.
    pub fn reserve_space(&mut self, reserve: bool) {self.layout.reserve = reserve;}

    /// Keep the panel present while a possible drag, captured drag, or settling animation owns it.
    pub fn is_dismissing(&self) -> bool {!matches!(self.drag, Drag::Idle)}

    /// Finish sliding away once the resting edge stops moving. An active gesture keeps ownership.
    pub fn dismiss(&mut self) {
        if matches!(self.drag, Drag::Possible {..} | Drag::Active {..}) {return}
        if matches!(self.drag, Drag::Settling {to, ..} if to > 0.0) {return}
        let height = self.layout.size.1;
        self.layout.drag_height = Some(height);
        self.drag = Drag::Settling {
            from: self.layout.offset, to: height,
            started: Instant::now() + Duration::from_millis(32),
        };
        self.timer.reset(Duration::from_millis(32));
    }

    pub fn reset(&mut self) {
        self.drag = Drag::Idle;
        self.timer.cancel();
        self.layout.offset = 0.0;
        self.layout.drag_height = None;
    }

    /// Advance an animation update, explicitly scheduling another only while settling.
    /// Returns true when dismissal finishes; the owner then hides the panel.
    pub fn advance(&mut self) -> bool {
        let Drag::Settling {from, to, started} = self.drag else {return false};
        let progress = (Instant::now().saturating_duration_since(started).as_secs_f32() / 0.22).min(1.0);
        self.layout.offset = from + (to - from) * (1.0 - (1.0 - progress).powi(3));
        if progress == 1.0 {
            self.drag = Drag::Idle;
            self.timer.cancel();
            if to == 0.0 {self.layout.drag_height = None;}
            return to > 0.0;
        }
        self.timer.reset(Duration::from_millis(16));
        false
    }

    /// Observe each parent event once, before passing it to children.
    ///
    /// `area` is the panel's resting rectangle in a stable event coordinate space; pass `None`
    /// while hidden. `handle` adds a drag-start rectangle in the same coordinate space, such
    /// as a toolbar above the panel. Convert moving local coordinates to window coordinates.
    /// Resting-edge movement absorbs drag motion and defers settling until geometry is quiet.
    pub fn observe(&mut self, event: &mut dyn DynEvent, area: Option<Area>, handle: Option<Area>) {
        if area.is_none() || matches!(event.downcast_ref::<WindowEvent>(),
            Some(WindowEvent::Focused(false))) {self.reset();}
        let now = Instant::now();
        let previous = self.area;
        self.area = area;
        if let Some(area) = area {
            let direction = match self.layout.edge {DismissEdge::Top => -1.0, DismissEdge::Bottom => 1.0};
            let height = area.size.1;
            match &mut self.drag {
                Drag::Active {anchor, last, velocity, ..} => {
                    self.layout.offset = ((last.0 - area.offset.1 - *anchor) * direction)
                        .clamp(0.0, height);
                    self.layout.drag_height = Some(height);
                    if self.layout.offset == 0.0 {*velocity = 0.0;}
                }
                Drag::Settling {from, to, started} => {
                    if *to > 0.0 {*to = height;}
                    self.layout.drag_height = Some(height);
                    self.layout.offset = self.layout.offset.min(height);
                    if let Some(previous) = previous {
                        let movement = (area.offset.1 - previous.offset.1) * direction;
                        if movement != 0.0 {
                            self.layout.offset = (self.layout.offset - movement.max(0.0))
                                .clamp(0.0, height);
                            *from = self.layout.offset;
                            // Leave room for the next resize frame before resuming our own slide.
                            *started = now + Duration::from_millis(32);
                            self.timer.reset(Duration::from_millis(32));
                        }
                    }
                    *from = from.min(height);
                }
                _ => {}
            }
        }
        let Some(mouse) = event.downcast_mut::<MouseEvent>() else {return};
        if matches!(mouse.state, MouseState::Pressed(_) | MouseState::Moved) {
            self.momentum = None;
        }
        let consumed = if matches!(mouse.state, MouseState::Scroll(..)) && self.momentum.is_some()
            && (mouse.position.is_none() || mouse.position == self.momentum) {true}
        else if let Some(area) = area {self.track(mouse, area, handle, now)}
        else {false};

        if consumed {
            mouse.position = None;
            if !matches!(mouse.state, MouseState::Released(_)) {mouse.state = MouseState::Moved;}
        }
    }

    fn track(&mut self, mouse: &MouseEvent, area: Area, handle: Option<Area>, now: Instant) -> bool {
        let direction = match self.layout.edge {DismissEdge::Top => -1.0, DismissEdge::Bottom => 1.0};
        let height = area.size.1;
        match mouse.state {
            MouseState::Pressed(MouseButton::Left) => {
                self.reset();
                let strip = height.min(12.0);
                let edge = Area {
                    offset: (area.offset.0, area.offset.1 + if direction < 0.0 {height - strip} else {0.0}),
                    size: (area.size.0, strip),
                };
                if let Some(origin) = mouse.position {
                    let inside = |area: Area| origin.0 >= area.offset.0
                        && origin.0 < area.offset.0 + area.size.0 && origin.1 >= area.offset.1
                        && origin.1 < area.offset.1 + area.size.1;
                    if inside(edge) || handle.is_some_and(inside) {
                        self.drag = Drag::Possible {
                            origin, anchor: origin.1 - area.offset.1 + direction * 8.0, started: now,
                        };
                    }
                }
            }
            MouseState::Moved | MouseState::Scroll(..) => {
                match self.drag {
                    Drag::Possible {origin, anchor, started} => {
                        let Some(position) = mouse.position else {
                            self.drag = Drag::Idle;
                            return false;
                        };
                        let dx = position.0 - origin.0;
                        let dy = (position.1 - origin.1) * direction;
                        if dx.abs() > 8.0 && dx.abs() >= dy.abs() || dy < -8.0 {
                            self.drag = Drag::Idle;
                        } else if dy > 8.0 && dy > dx.abs() {
                            self.layout.drag_height = Some(height);
                            self.layout.offset = ((position.1 - area.offset.1 - anchor) * direction)
                                .clamp(0.0, height);
                            self.drag = Drag::Active {
                                origin, anchor, last: (position.1, now),
                                velocity: if self.layout.offset == 0.0 {0.0}
                                    else {dy / now.duration_since(started).as_secs_f32().max(0.001)},
                            };
                            return true;
                        }
                    }
                    Drag::Active {origin, anchor, last, velocity} => {
                        let position = mouse.position.map(|p| p.1).or({
                            if let MouseState::Scroll(_, dy) = mouse.state {Some(last.0 - dy)}
                            else {None}
                        });
                        if let Some(y) = position {
                            let elapsed = now.duration_since(last.1).as_secs_f32().max(0.001);
                            self.layout.offset = ((y - area.offset.1 - anchor) * direction)
                                .clamp(0.0, height);
                            self.drag = Drag::Active {
                                origin, anchor, last: (y, now),
                                velocity: if self.layout.offset == 0.0 {0.0} else if y == last.0 {velocity}
                                    else {(y - last.0) * direction / elapsed},
                            };
                        } else {
                            self.drag = Drag::Settling {from: self.layout.offset, to: 0.0, started: now};
                            self.timer.reset(Duration::from_millis(16));
                        }
                        return true;
                    }
                    Drag::Settling {..} => return true,
                    Drag::Idle => {}
                }
            }
            MouseState::Released(MouseButton::Left) => {
                let drag = std::mem::replace(&mut self.drag, Drag::Idle);
                if let Drag::Active {origin, anchor, last, velocity} = drag {
                    if let Some(position) = mouse.position {
                        self.layout.offset = ((position.1 - area.offset.1 - anchor) * direction)
                            .clamp(0.0, height);
                    }
                    let velocity = if now.duration_since(last.1).as_secs_f32() < 0.1 {
                        velocity
                    } else {0.0};
                    let close = self.layout.offset > 0.0 && (velocity >= 600.0
                        || self.layout.offset >= height * 0.4 && velocity > -200.0);
                    self.drag = Drag::Settling {
                        from: self.layout.offset, to: if close {height} else {0.0},
                        started: now + if close {Duration::from_millis(32)} else {Duration::ZERO},
                    };
                    self.timer.reset(Duration::from_millis(16));
                    self.momentum = Some(origin);
                    return true;
                }
                if let Drag::Settling {..} = drag {
                    self.drag = drag;
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

#[on_change(self.timer.next())]
impl<D: Drawable + Clone> OnChange<()> for Dismissible<D> {
    fn on_change(&mut self, _ctx: &mut Context, _: ()) {}
}

#[derive(Debug, Clone, Copy)]
enum Drag {
    Idle,
    Possible {origin: (f32, f32), anchor: f32, started: Instant},
    Active {origin: (f32, f32), anchor: f32, last: (f32, Instant), velocity: f32},
    Settling {from: f32, to: f32, started: Instant},
}

#[derive(Debug, Clone)]
struct SlideLayout {
    edge: DismissEdge,
    size: (f32, f32),
    child: SizeRange,
    offset: f32,
    reserve: bool,
    drag_height: Option<f32>,
}

impl Layout for SlideLayout {
    fn request_width(&mut self, children: Vec<(f32, f32)>) -> (f32, f32) {
        let (min_width, max_width) = children[0];
        self.child = SizeRange {min_width, max_width, ..SizeRange::default()};
        (min_width, max_width)
    }

    fn assign_width(&mut self, width: f32) -> Vec<f32> {
        let width = width.clamp(self.child.min_width, self.child.max_width);
        self.child.min_width = width;
        self.child.max_width = width;
        vec![width]
    }

    fn request_height(&mut self, _width: f32, children: Vec<(f32, f32)>) -> (f32, f32) {
        let (min, max) = children[0];
        self.child.min_height = min;
        self.child.max_height = max;
        if !self.reserve && let Some(height) = self.drag_height {
            let height = (height - self.offset).max(0.0);
            (height, height)
        } else {(min, max)}
    }

    fn layout(&mut self, size: (f32, f32)) -> Vec<Area> {
        let full_size = (size.0, size.1 + if self.reserve {0.0} else {self.offset});
        self.size = full_size;
        let y = match self.edge {
            DismissEdge::Top => -self.offset,
            DismissEdge::Bottom if self.reserve => self.offset,
            DismissEdge::Bottom => 0.0,
        };
        vec![Area {offset: (0.0, y), size: self.child.clamp(full_size)}]
    }
}
