use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{
    DynEvent, Key, KeyboardEvent, KeyboardState, MouseButton, MouseEvent, MouseState, OnEvent,
    WindowEvent, on_event,
};
use uuid::Uuid;


use crate::events as event;
use crate::layout::{Column, Offset, Padding, ScrollAnchor, Size, Stack};
use crate::navigation::NavigationChanged;

/// Buttons react to presses; scroll containers decide when a press is a tap.
#[derive(Debug, Component, Clone)]
#[events(MouseEvent, WindowEvent, event::Press)]
pub struct Button<D: Drawable + Clone>(Stack, pub D, #[skip] bool, #[skip] Press);
impl<D: Drawable + Clone> Button<D> {
    pub fn new(child: D) -> Self {Self(Stack::default(), child, false, Press::default())}
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<MouseEvent> for Button<D> {
    fn on(&mut self, ctx: &mut Context, mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        match mouse.state {
            MouseState::Pressed(MouseButton::Left) if mouse.position.is_some() => {
                self.3.clear();
                self.2 = true;
                return prism::events![event::Button::Pressed(true)];
            }
            MouseState::Released(MouseButton::Left) => {
                self.3.clear();
                let pressed = std::mem::take(&mut self.2);
                if pressed && mouse.position.is_some() {
                    return prism::events![event::Button::Pressed(false)];
                }
                return prism::events![event::Button::Hover(false)];
            }
            MouseState::Moved if !ctx.window.is_mobile => {
                return prism::events![event::Button::Hover(mouse.position.is_some())];
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
            self.2 = false;
            self.3.clear();
            return prism::events![event::Button::Hover(false), incoming];
        }
        prism::events![incoming]
    }
}

#[on_event]
impl<D: Drawable + Clone> OnEvent<event::Press> for Button<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        match incoming {
            event::Press::Pending {id, position} => {
                self.3.pending(id, position);
                self.2 = false;
                if position.is_some() {return vec![];}
            }
            event::Press::Cancel(id) if self.3.cancel(id) => {
                self.2 = false;
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
    fn on(&mut self, _ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        match incoming {
            event::Press::Pending {id, position} => self.4.pending(id, position),
            event::Press::Commit(_) => {},
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
        if let event::TextInput::Focused(focused) = incoming
            && let Some(focus) = &mut self.2 {*focus = focused;}
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
    fn on(&mut self, _ctx: &mut Context, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        match incoming {
            event::Press::Pending {id, position} => self.3.pending(id, position),
            event::Press::Commit(_) => {},
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
#[events(event::Scrollable, MouseEvent, WindowEvent, event::Press, NavigationChanged)]
pub struct Scrollable<D: Drawable + Clone>(Column, pub D, #[skip] DeferredPress);
impl<D: Drawable + Clone> Scrollable<D> {
    pub fn scroll(&self) -> f32 {self.0.scroll()}
    pub fn set_scroll(&mut self, offset: f32) {self.0.set_scroll(offset);}
    pub fn adjust_scroll(&mut self, delta: f32) {self.0.adjust_scroll(delta);}

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
        let mut events = self.2.mouse(ctx.window.is_mobile, incoming);
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
            events.retain(|event| !matches!(event.downcast_ref::<MouseEvent>(),
                Some(MouseEvent {state: MouseState::Scroll(..), ..})));
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

#[on_event]
impl<D: Drawable + Clone> OnEvent<NavigationChanged> for Scrollable<D> {
    fn on(&mut self, _ctx: &mut Context, incoming: NavigationChanged) -> Vec<Box<dyn DynEvent>> {
        let mut events = Vec::new();
        if let Some(cancel) = self.2.cancel() {events.push(Box::new(cancel) as Box<dyn DynEvent>);}
        events.push(Box::new(incoming));
        events
    }
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

/// Gesture arbitration shared by vertical lists and horizontal media pickers.
/// No timeout can prove a tap: an unclaimed gesture commits only when it ends.
#[derive(Debug, Default)]
pub struct DeferredPress {
    target: Option<(Uuid, bool)>,
    gesture: Option<Gesture>,
    confirmed: bool,
}

impl Clone for DeferredPress {
    // A cloned page is not participating in the original page's touch sequence.
    fn clone(&self) -> Self {Self::default()}
}

#[derive(Debug, Clone)]
struct Gesture {
    origin: (f32, f32),
    travel: (f32, f32),
    scroll: (f32, f32),
}

impl DeferredPress {
    const TAP_SLOP: f32 = 6.0;

    pub fn mouse(&mut self, mobile: bool, mut mouse: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        let mut events: Vec<Box<dyn DynEvent>> = Vec::new();
        // An ancestor already resolved this gesture. Bypass only its replayed
        // press, not later hardware input if a page changes before release.
        if std::mem::take(&mut self.confirmed)
            && mouse.state == MouseState::Pressed(MouseButton::Left)
        {
            return prism::events![mouse];
        }
        if mouse.state == MouseState::Pressed(MouseButton::Left) {
            if let Some(cancel) = self.cancel() {events.push(Box::new(cancel));}
            if let Some(position) = mouse.position {
                let id = Uuid::new_v4();
                self.begin(id, position);
                events.push(Box::new(event::Press::Pending {id, position: Some(position)}));
                return events;
            }
        }

        // A slider can explicitly claim a pending gesture. Ordinary buttons,
        // fields and rows cannot steal a gesture while scrolling is undecided.
        if self.claimed() {
            if mouse.state == MouseState::Released(MouseButton::Left) {
                self.target = None;
                self.gesture = None;
            }
            return prism::events![mouse];
        }

        if matches!(mouse.state, MouseState::Moved | MouseState::Scroll(..)
            | MouseState::Released(MouseButton::Left))
            && let Some(gesture) = &mut self.gesture
        {
            if let Some((x, y)) = mouse.position {
                gesture.travel = (x - gesture.origin.0, y - gesture.origin.1);
            }
            let wheel = if let MouseState::Scroll(x, y) = mouse.state {
                gesture.scroll.0 += x;
                gesture.scroll.1 += y;
                x != 0.0 || y != 0.0
            } else {false};
            let slop = if mobile {8.0} else {Self::TAP_SLOP};
            let moved = gesture.travel.0.hypot(gesture.travel.1) > slop
                || gesture.scroll.0.hypot(gesture.scroll.1) > slop;
            // iOS reports even subpixel finger movement as Scroll. Accumulate it
            // without moving the list or activating a row until intent is clear.
            if (mouse.position.is_none() || moved || (!mobile && wheel))
                && let Some((id, _)) = self.target.take()
            {
                // Cancellation is sticky, even if the pointer returns to its origin.
                events.push(Box::new(event::Press::Cancel(id)));
                if matches!(mouse.state, MouseState::Scroll(..)) {
                    mouse.state = MouseState::Scroll(gesture.scroll.0, gesture.scroll.1);
                }
            }
            if mouse.state == MouseState::Released(MouseButton::Left) {
                if let Some((id, _)) = self.target.take() {
                    events.push(Box::new(event::Press::Commit(id)));
                    events.push(Box::new(MouseEvent {
                        position: Some(gesture.origin), state: MouseState::Pressed(MouseButton::Left),
                    }));
                    events.push(Box::new(mouse));
                }
                self.gesture = None;
            } else if self.target.is_none() && matches!(mouse.state, MouseState::Scroll(..)) {
                // Only the owning scroll container consumes this, not its controls.
                events.push(Box::new(mouse));
            }
            return events;
        }
        events.push(Box::new(mouse));
        events
    }

    fn begin(&mut self, id: Uuid, position: (f32, f32)) {
        self.target = Some((id, false));
        self.gesture = Some(Gesture {origin: position, travel: (0.0, 0.0), scroll: (0.0, 0.0)});
    }

    pub fn press(&mut self, incoming: event::Press) -> Vec<Box<dyn DynEvent>> {
        let mut events: Vec<Box<dyn DynEvent>> = Vec::new();
        match incoming {
            event::Press::Pending {id, position} => {
                if let Some(cancel) = self.cancel() {events.push(Box::new(cancel));}
                if let Some(position) = position {self.begin(id, position);}
            }
            event::Press::Commit(id) | event::Press::Cancel(id) => {
                if self.target.is_some_and(|(target, _)| target == id) {
                    self.target = None;
                    self.gesture = None;
                    self.confirmed = matches!(incoming, event::Press::Commit(_));
                }
            }
            event::Press::Claim(id) => {
                if let Some((target, claimed)) = self.target.as_mut() && *target == id {
                    *claimed = true;
                }
            }
        }
        events.push(Box::new(incoming));
        events
    }

    pub fn cancel(&mut self) -> Option<event::Press> {
        self.confirmed = false;
        self.gesture = None;
        self.target.take().map(|(id, _)| event::Press::Cancel(id))
    }

    pub fn travel(&self) -> Option<(f32, f32)> {self.gesture.as_ref().map(|gesture| gesture.travel)}
    pub fn claimed(&self) -> bool {self.target.is_some_and(|(_, claimed)| claimed)}
}
