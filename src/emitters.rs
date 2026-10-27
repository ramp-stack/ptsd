use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{
    DynEvent, Key, KeyboardEvent, KeyboardState, MouseButton, MouseEvent, MouseState, OnEvent,
    WindowEvent, OnChange, Timer, on_change, on_event,
};
use uuid::Uuid;

use std::time::{Duration, Instant};

use crate::events as event;
use crate::layout::{Column, Offset, Padding, ScrollAnchor, Size, Stack};

#[derive(Debug, Component, Clone)]
#[events(MouseEvent, WindowEvent, event::Press)]
pub struct Button<D: Drawable + Clone>(
    Stack, pub D, #[skip] Option<PressState>, #[skip] Press,
);
impl<D: Drawable + Clone> Button<D> {
    pub fn new(child: D) -> Self {Self(Stack::default(), child, None, Press::default())}
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<MouseEvent> for Button<D> {
    fn on(&mut self, ctx: &mut Context, mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let mobile = ctx.window.is_mobile;
        match mouse.state {
            MouseState::Pressed(MouseButton::Left) if mouse.position.is_some() => {
                self.3.clear();
                let (x, y) = mouse.position.unwrap();
                self.2 = Some(PressState {
                    started: Instant::now(), origin: (x as f64, y as f64), cancelled: false,
                });
                return prism::events![event::Button::Pressed(true)];
            }
            MouseState::Moved | MouseState::Scroll(..) if mobile && self.2.is_some() => {
                let press = self.2.as_mut().unwrap();
                press.cancelled |= match (mouse.state, mouse.position) {
                    (MouseState::Moved | MouseState::Scroll(..), Some((x, y))) => {
                        (x as f64 - press.origin.0).powi(2)
                            + (y as f64 - press.origin.1).powi(2) > 12.0_f64.powi(2)
                    }
                    _ => true,
                };
            }
            MouseState::Moved | MouseState::Scroll(..) if !mobile => {
                return prism::events![event::Button::Hover(mouse.position.is_some())];
            }
            MouseState::Released(MouseButton::Left) => {
                self.3.clear();
                let Some(press) = self.2.take() else {return prism::events![mouse]};
                let valid = press.started.elapsed() <= Duration::from_millis(600)
                    && !press.cancelled && mouse.position.is_some();
                return match (mobile, valid, mouse.position.is_some()) {
                    (true, true, _) => prism::events![event::Button::Pressed(false)],
                    (true, false, _) => prism::events![event::Button::Hover(false)],
                    (false, _, true) => prism::events![
                        event::Button::Pressed(false), event::Button::Hover(true),
                    ],
                    _ => prism::events![event::Button::Hover(false)],
                };
            }
            _ => {}
        }
        prism::events![mouse]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<WindowEvent> for Button<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(incoming, WindowEvent::Focused(false)) {
            self.2 = None;
            self.3.clear();
            return prism::events![event::Button::Hover(false), incoming];
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Press> for Button<D> {
    fn on(&mut self, ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        match incoming {
            event::Press::Pending {id, position} => {
                self.3.pending(id, position);
                self.2 = position.map(|(x, y)| PressState {
                    started: Instant::now(), origin: (x as f64, y as f64), cancelled: false,
                });
                if position.is_some() {return vec![]}
            }
            event::Press::Commit(id) if self.3.commit(id).is_some() => {
                ctx.emit(event::Press::Claim(id));
                return prism::events![event::Button::Pressed(true)];
            }
            event::Press::Cancel(id) if self.3.cancel(id) => {
                self.2 = None;
                return prism::events![event::Button::Hover(false), incoming];
            }
            _ => {}
        }
        prism::events![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[events(KeyboardEvent)]
pub struct NumericalInput<D: Drawable + Clone>(Stack, pub D);
impl<D: Drawable + Clone> NumericalInput<D> {
    pub fn new(child: D) -> Self {Self(Stack::default(), child)}
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<KeyboardEvent> for NumericalInput<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: KeyboardEvent) -> Vec<Box<dyn DynEvent>> {
        if incoming.state == KeyboardState::Pressed {
            match &incoming.key {
                Key::Delete | Key::Backspace => {
                    return prism::events![event::NumericalInput::Delete];
                }
                Key::Character(text) => {
                    if let Some(ch) = text.chars().next() {
                        if ch.is_ascii_digit() {return prism::events![event::NumericalInput::Digit(ch)]}
                        if matches!(ch, '.' | '/' | ':') {
                            return prism::events![event::NumericalInput::Char(ch)];
                        }
                    }
                }
                _ => {}
            }
        }
        prism::events![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[events(MouseEvent, WindowEvent, event::Selectable, event::Press)]
pub struct Selectable<D: Drawable + Clone>(
    Stack, pub D, #[skip] Uuid, #[skip] Uuid, #[skip] Press,
);
impl<D: Drawable + Clone> Selectable<D> {
    pub fn new(child: D, group_id: Uuid) -> Self {
        Self(Stack::default(), child, Uuid::new_v4(), group_id, Press::default())
    }

    pub fn select(&self, ctx: &mut Context) {
        ctx.emit(event::Selectable::Pressed(self.2.to_string(), self.3.to_string()));
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<MouseEvent> for Selectable<D> {
    fn on(&mut self, ctx: &mut Context, incoming: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(incoming.state, MouseState::Pressed(_) | MouseState::Released(_)) {self.4.clear();}
        if let MouseEvent {position: Some(_), state: MouseState::Pressed(MouseButton::Left)} = incoming {
            self.select(ctx);
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Press> for Selectable<D> {
    fn on(&mut self, ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        match incoming {
            event::Press::Pending {id, position} => self.4.pending(id, position),
            event::Press::Commit(id) => if self.4.commit(id).is_some() {
                ctx.emit(event::Press::Claim(id));
                self.select(ctx);
            },
            event::Press::Claim(_) => {}
            event::Press::Cancel(id) => {self.4.cancel(id);}
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<WindowEvent> for Selectable<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(incoming, WindowEvent::Focused(false)) {self.4.clear();}
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Selectable> for Selectable<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: event::Selectable) -> Vec<Box<dyn DynEvent>> {
        if let event::Selectable::Pressed(id, group_id) = &incoming && *group_id == self.3.to_string() {
            return prism::events![event::Selectable::Selected(*id == self.2.to_string())];
        }
        prism::events![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[events(MouseEvent, WindowEvent, event::Press)]
pub struct Slider<D: Drawable + Clone>(Stack, pub D, #[skip] bool, #[skip] Press);
impl<D: Drawable + Clone> Slider<D> {
    pub fn new(child: D) -> Self {Self(Stack::default(), child, false, Press::default())}
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<MouseEvent> for Slider<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        match (incoming.state, incoming.position) {
            (MouseState::Pressed(MouseButton::Left), Some((x, _))) => {
                self.3.clear();
                self.2 = true;
                prism::events![event::Slider::Start(x)]
            }
            (MouseState::Released(MouseButton::Left), _) => {
                self.2 = false;
                self.3.clear();
                prism::events![incoming]
            }
            (MouseState::Scroll(..) | MouseState::Moved, Some((x, _))) if self.2 => {
                prism::events![event::Slider::Moved(x)]
            }
            _ => prism::events![incoming],
        }
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Press> for Slider<D> {
    fn on(&mut self, ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        match incoming {
            event::Press::Pending {id, position} => {
                self.3.pending(id, position);
                if let Some((x, _)) = self.3.commit(id) {
                    self.2 = true;
                    ctx.emit(event::Press::Claim(id));
                    return prism::events![event::Slider::Start(x)];
                }
            }
            event::Press::Commit(_) | event::Press::Claim(_) => {}
            event::Press::Cancel(id) => if self.3.cancel(id) {self.2 = false;},
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<WindowEvent> for Slider<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(incoming, WindowEvent::Focused(false)) {self.2 = false; self.3.clear();}
        prism::events![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[events(event::Selectable, event::TextInput, MouseEvent, KeyboardEvent, WindowEvent, event::Press)]
pub struct TextInput<D: Drawable + Clone>(Stack, pub D, #[skip] Option<bool>, #[skip] Press);
impl<D: Drawable + Clone> TextInput<D> {
    pub fn new(child: D, requires_focus: bool) -> Selectable<Self> {
        Selectable::new(
            Self(Stack::default(), child, requires_focus.then_some(false), Press::default()),
            uuid::uuid!("123e4567-e89b-12d3-a456-426614174000"),
        )
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Selectable> for TextInput<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: event::Selectable) -> Vec<Box<dyn DynEvent>> {
        if let event::Selectable::Selected(selected) = incoming {
            if let Some(focus) = &mut self.2 {*focus = selected;}
            return prism::events![event::TextInput::Focused(selected), incoming];
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::TextInput> for TextInput<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: event::TextInput) -> Vec<Box<dyn DynEvent>> {
        if let event::TextInput::Focused(focused) = incoming {
            if let Some(focus) = &mut self.2 {*focus = focused;}
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<MouseEvent> for TextInput<D> {
    fn on(&mut self, ctx: &mut Context, mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let mobile = ctx.window.is_mobile;
        if matches!(mouse.state, MouseState::Pressed(_) | MouseState::Released(_)) {self.3.clear();}
        let mut events: Vec<Box<dyn DynEvent>> = vec![];
        match mouse.state {
            MouseState::Pressed(MouseButton::Left) if mouse.position.is_some() => {
                if let Some(focus) = &mut self.2 {*focus = true;}
                events.push(Box::new(event::TextInput::Focused(true)));
            }
            MouseState::Pressed(MouseButton::Left) if mouse.position.is_none() && !mobile => {
                if let Some(focus) = &mut self.2 {*focus = false;}
                events.push(Box::new(event::TextInput::Focused(false)));
            }
            MouseState::Moved | MouseState::Scroll(..) if !mobile && !self.2.unwrap_or_default() => {
                events.push(Box::new(event::TextInput::Hover(mouse.position.is_some())));
            }
            _ => {}
        }
        events.push(Box::new(mouse));
        events
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Press> for TextInput<D> {
    fn on(&mut self, ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        match incoming {
            event::Press::Pending {id, position} => self.3.pending(id, position),
            event::Press::Commit(id) => if self.3.commit(id).is_some() {
                ctx.emit(event::Press::Claim(id));
                if let Some(focus) = &mut self.2 {*focus = true;}
                return prism::events![event::TextInput::Focused(true), incoming];
            },
            event::Press::Claim(_) => {}
            event::Press::Cancel(id) => {self.3.cancel(id);}
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<WindowEvent> for TextInput<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(incoming, WindowEvent::Focused(false)) {self.3.clear();}
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<KeyboardEvent> for TextInput<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: KeyboardEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(incoming.state, KeyboardState::Pressed | KeyboardState::Repeated) {
            let key = incoming.key.clone();
            return match self.2 {
                Some(false) => vec![],
                _ => prism::events![incoming, event::TextInput::Edited(key)],
            };
        }
        prism::events![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[events(event::Scrollable, MouseEvent, WindowEvent, event::Press)]
pub struct Scrollable<D: Drawable + Clone>(Column, pub D, #[skip] DeferredPress);
impl<D: Drawable + Clone> Scrollable<D> {
    pub fn new(child: D) -> Self {
        let layout = Column::new(
            0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::Start),
        );
        Self(layout, child, DeferredPress::default())
    }

    pub fn end(child: D) -> Self {
        let layout = Column::new(
            0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::End),
        );
        Self(layout, child, DeferredPress::default())
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Scrollable> for Scrollable<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: event::Scrollable) -> Vec<Box<dyn DynEvent>> {
        let event::Scrollable::ScrollTo(y) = incoming;
        self.0.set_scroll(y);
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<MouseEvent> for Scrollable<D> {
    fn on(&mut self, ctx: &mut Context, incoming: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let events = self.2.mouse(ctx.window.is_mobile, incoming);
        for incoming in &events {
            if let Some(event::Press::Cancel(id)) = incoming.downcast_ref() {
                ctx.emit(event::Press::Cancel(*id));
            }
        }
        if !self.2.claimed() {
            for incoming in &events {
                if let Some(MouseEvent {state: MouseState::Scroll(_, y), position: Some(_)}) =
                    incoming.downcast_ref() {self.0.adjust_scroll(*y);}
            }
        }
        events
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Press> for Scrollable<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        self.2.press(incoming)
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<WindowEvent> for Scrollable<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        let mut events = Vec::new();
        if matches!(incoming, WindowEvent::Focused(false)) && let Some(cancel) = self.2.cancel() {
            events.push(Box::new(cancel) as Box<dyn DynEvent>);
        }
        events.push(Box::new(incoming));
        events
    }
}

#[on_change(self.2.next())]
impl<D: Drawable + Clone> OnChange<event::Press> for Scrollable<D> {
    fn on_change(&mut self, ctx: &mut Context, incoming: event::Press) {ctx.emit(incoming);}
}

#[derive(Debug, Default, Clone)]
pub struct Press {
    captured: Option<(Uuid, Option<(f32, f32)>)>,
}

impl Press {
    pub fn pending(&mut self, id: Uuid, position: Option<(f32, f32)>) {
        self.captured = position.map(|position| (id, Some(position)));
    }

    pub fn commit(&mut self, id: Uuid) -> Option<(f32, f32)> {
        let (captured, position) = self.captured.as_mut()?;
        if *captured != id {return None}
        position.take()
    }

    pub fn cancel(&mut self, id: Uuid) -> bool {
        if self.captured.is_some_and(|(captured, _)| captured == id) {
            self.clear();
            true
        } else {false}
    }

    pub fn clear(&mut self) {self.captured = None;}
}

#[derive(Debug, Default, Clone)]
pub struct DeferredPress {
    timer: Timer,
    target: Option<(Uuid, bool)>,
    gesture: Option<((f32, f32), (f32, f32))>,
}

impl DeferredPress {
    pub fn mouse(&mut self, mobile: bool, mut mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let mut events = Vec::new();
        if mouse.state == MouseState::Pressed(MouseButton::Left) {
            if let Some(cancel) = self.cancel() {events.push(Box::new(cancel) as Box<dyn DynEvent>);}
            self.gesture = mouse.position.map(|position| (position, (0.0, 0.0)));
            if mobile && mouse.position.is_some() {
                let id = Uuid::new_v4();
                self.target = Some((id, false));
                self.timer.reset(Duration::from_millis(150));
                events.push(Box::new(event::Press::Pending {id, position: mouse.position}));
                return events;
            }
        }

        if matches!(mouse.state, MouseState::Moved | MouseState::Scroll(..)
            | MouseState::Released(MouseButton::Left)) {
            if let Some(travel) = self.travel_at(mouse.position) {
                self.gesture.as_mut().unwrap().1 = travel;
            }
            if let Some((id, false)) = self.target {
                let travel = self.travel().unwrap();
                if mouse.position.is_none() || travel.0.powi(2) + travel.1.powi(2) > 12.0_f32.powi(2) {
                    self.target = None;
                    self.timer.cancel();
                    events.push(Box::new(event::Press::Cancel(id)) as Box<dyn DynEvent>);
                    if mouse.position.is_some() && matches!(mouse.state, MouseState::Scroll(..)) {
                        mouse.state = MouseState::Scroll(-travel.0, -travel.1);
                    }
                } else if matches!(mouse.state, MouseState::Scroll(..)) {return events}
            }
            if mouse.position.is_none() {self.gesture = None;}
        }

        if mouse.state == MouseState::Released(MouseButton::Left) {
            if let Some((id, false)) = self.target.take() {
                events.push(Box::new(event::Press::Commit(id)) as Box<dyn DynEvent>);
            }
            self.timer.cancel();
            self.gesture = None;
        }
        events.push(Box::new(mouse));
        events
    }

    pub fn press(&mut self, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        let mut events = Vec::new();
        match incoming {
            event::Press::Pending {id, position} => {
                if let Some(cancel) = self.cancel() {events.push(Box::new(cancel) as Box<dyn DynEvent>);}
                self.target = position.map(|_| (id, false));
                self.gesture = position.map(|position| (position, (0.0, 0.0)));
            }
            event::Press::Commit(id) => {
                if self.target.is_some_and(|(target, _)| target == id) {
                    self.timer.cancel();
                }
            }
            event::Press::Claim(id) => {
                if let Some((target, claimed)) = self.target.as_mut() && *target == id {
                    *claimed = true;
                    self.timer.cancel();
                }
            }
            event::Press::Cancel(id) => {
                if self.target.is_some_and(|(target, _)| target == id) {
                    self.target = None;
                    self.timer.cancel();
                }
            }
        }
        events.push(Box::new(incoming));
        events
    }

    pub fn cancel(&mut self) -> Option<event::Press> {
        self.timer.cancel();
        self.gesture = None;
        self.target.take().map(|(id, _)| event::Press::Cancel(id))
    }

    pub async fn next(&mut self) -> event::Press {
        self.timer.next().await;
        event::Press::Commit(self.target.unwrap().0)
    }

    pub fn travel(&self) -> Option<(f32, f32)> {self.gesture.map(|(_, travel)| travel)}
    pub fn travel_at(&self, position: Option<(f32, f32)>) -> Option<(f32, f32)> {
        let ((x, y), _) = self.gesture?;
        position.map(|position| (position.0 - x, position.1 - y))
    }
    pub fn claimed(&self) -> bool {self.target.is_some_and(|(_, claimed)| claimed)}
}

#[derive(Debug, Clone)]
struct PressState {
    started: Instant,
    origin: (f64, f64),
    cancelled: bool,
}
