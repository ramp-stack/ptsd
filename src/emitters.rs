use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{
    DynEvent, Key, KeyboardEvent, KeyboardState, MouseButton, MouseEvent, MouseState, WindowEvent,
};

use std::time::{Duration, Instant};

use crate::events as event;
use crate::layout::{Column, Offset, Padding, ScrollAnchor, Size, Stack};

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct Button<D: Drawable + Clone>(Stack, pub D, #[skip] Option<PressState>);
impl<D: Drawable + Clone> Button<D> {
    pub fn new(child: D) -> Self {Self(Stack::default(), child, None)}

    fn on_event(&mut self, ctx: &mut Context, incoming: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let mobile = ctx.window.touch_controls;
        if matches!(incoming.downcast_ref::<WindowEvent>(), Some(WindowEvent::Focused(false))) {
            self.2 = None;
            return vec![Box::new(event::Button::Hover(false)), incoming];
        }
        if let Some(mouse) = incoming.downcast_ref::<MouseEvent>() {
            match mouse.state {
                MouseState::Pressed(MouseButton::Left) if mouse.position.is_some() => {
                    let (x, y) = mouse.position.unwrap();
                    self.2 = Some(PressState {
                        started: Instant::now(), origin: (x as f64, y as f64), cancelled: false,
                    });
                    return prism::events![event::Button::Pressed(true)];
                }
                MouseState::Moved | MouseState::Scroll(..) if mobile && self.2.is_some() => {
                    let press = self.2.as_mut().unwrap();
                    press.cancelled |= match (mouse.state, mouse.position) {
                        (MouseState::Moved, Some((x, y))) => {
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
                    let Some(press) = self.2.take() else {return vec![]};
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
        }
        vec![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct NumericalInput<D: Drawable + Clone>(Stack, pub D);
impl<D: Drawable + Clone> NumericalInput<D> {
    pub fn new(child: D) -> Self {Self(Stack::default(), child)}

    fn on_event(&mut self, _ctx: &mut Context, incoming: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(KeyboardEvent {state: KeyboardState::Pressed, key, ..}) = incoming.downcast_ref() {
            match key {
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
        vec![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct Selectable<D: Drawable + Clone>(Stack, pub D, #[skip] uuid::Uuid, #[skip] uuid::Uuid);
impl<D: Drawable + Clone> Selectable<D> {
    pub fn new(child: D, group_id: uuid::Uuid) -> Self {
        Self(Stack::default(), child, uuid::Uuid::new_v4(), group_id)
    }

    pub fn select(&self, ctx: &mut Context) {
        ctx.emit(event::Selectable::Pressed(self.2.to_string(), self.3.to_string()));
    }

    fn on_event(&mut self, ctx: &mut Context, incoming: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(MouseEvent {position: Some(_), state: MouseState::Pressed(MouseButton::Left)}) =
            incoming.downcast_ref::<MouseEvent>() {
            self.select(ctx);
        } else if let Some(event::Selectable::Pressed(id, group_id)) = incoming.downcast_ref()
            && *group_id == self.3.to_string() {
            return prism::events![event::Selectable::Selected(*id == self.2.to_string())];
        }
        vec![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct Slider<D: Drawable + Clone>(Stack, pub D, #[skip] bool);
impl<D: Drawable + Clone> Slider<D> {
    pub fn new(child: D) -> Self {Self(Stack::default(), child, false)}

    fn on_event(&mut self, _ctx: &mut Context, incoming: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(MouseEvent {state, position}) = incoming.downcast_ref() {
            return match (state, position) {
                (MouseState::Pressed(MouseButton::Left), Some((x, _))) => {
                    self.2 = true;
                    prism::events![event::Slider::Start(*x)]
                }
                (MouseState::Released(MouseButton::Left), _) => {
                    self.2 = false;
                    vec![]
                }
                (MouseState::Scroll(..) | MouseState::Moved, Some((x, _))) if self.2 => {
                    prism::events![event::Slider::Moved(*x)]
                }
                _ => vec![],
            };
        }
        vec![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct TextInput<D: Drawable + Clone>(Stack, pub D, #[skip] Option<bool>);
impl<D: Drawable + Clone> TextInput<D> {
    pub fn new(child: D, requires_focus: bool) -> Selectable<Self> {
        Selectable::new(
            Self(Stack::default(), child, requires_focus.then_some(false)),
            uuid::uuid!("123e4567-e89b-12d3-a456-426614174000"),
        )
    }

    fn on_event(&mut self, ctx: &mut Context, incoming: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        let mobile = ctx.window.touch_controls;
        if let Some(event::Selectable::Selected(selected)) = incoming.downcast_ref() {
            if let Some(focus) = &mut self.2 {*focus = *selected;}
            return vec![Box::new(event::TextInput::Focused(*selected)), incoming];
        } else if let Some(mouse) = incoming.downcast_ref::<MouseEvent>() {
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
            events.push(incoming);
            return events;
        } else if let Some(KeyboardEvent {state: KeyboardState::Pressed | KeyboardState::Repeated, key, ..}) =
            incoming.downcast_ref() {
            let key = key.clone();
            return match self.2 {
                Some(false) => vec![],
                _ => vec![incoming, Box::new(event::TextInput::Edited(key))],
            };
        }
        vec![incoming]
    }
}

#[derive(Debug, Component, Clone)]
#[on_dyn_event]
pub struct Scrollable<D: Drawable + Clone>(Column, pub D);
impl<D: Drawable + Clone> Scrollable<D> {
    pub fn new(child: D) -> Self {
        let layout = Column::new(
            0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::Start),
        );
        Self(layout, child)
    }

    pub fn end(child: D) -> Self {
        let layout = Column::new(
            0.0, Offset::Start, Size::Fit, Padding::default(), Some(ScrollAnchor::End),
        );
        Self(layout, child)
    }

    fn on_event(&mut self, _ctx: &mut Context, incoming: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(event::Scrollable::ScrollTo(y)) = incoming.downcast_ref() {
            self.0.set_scroll(*y);
        }
        if let Some(MouseEvent {state: MouseState::Scroll(_, y), position: Some(_)}) =
            incoming.downcast_ref() {
            self.0.adjust_scroll(*y);
        }
        vec![incoming]
    }
}

#[derive(Debug, Clone)]
struct PressState {
    started: Instant,
    origin: (f64, f64),
    cancelled: bool,
}
