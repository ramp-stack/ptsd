use std::time::{Duration, Instant};

use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{on_event, DynEvent, MouseButton, MouseEvent, MouseState, OnEvent, TickEvent};

use crate::display::Enum;
use crate::emitters;
use crate::events as event;
use crate::layout::Stack;
use crate::utils::Callback;

#[derive(Component, Debug, Clone)]
pub struct Button(Stack, emitters::Button<_Button>, #[skip] Option<Option<(f32, f32)>>);
impl Button {
    pub fn new(
        default: impl Drawable, hover: Option<impl Drawable>, pressed: Option<impl Drawable>,
        disabled: Option<impl Drawable>, feedback: Option<impl Drawable>,
        callback: impl FnMut(&mut Context) + Clone + 'static, disableable: bool,
    ) -> Self {
        let button = _Button::new(
            default, hover, pressed, disabled, feedback, callback, disableable, false,
        );
        Self(Stack::default(), emitters::Button::new(button), None)
    }

    pub fn new_triggers_on_release(
        default: impl Drawable, hover: Option<impl Drawable>, pressed: Option<impl Drawable>,
        disabled: Option<impl Drawable>, feedback: Option<impl Drawable>,
        callback: impl FnMut(&mut Context) + Clone + 'static, disableable: bool,
    ) -> Self {
        let button = _Button::new(
            default, hover, pressed, disabled, feedback, callback, disableable, true,
        );
        Self(Stack::default(), emitters::Button::new(button), None)
    }

    pub fn set_trigger_on_release(&mut self) {self.1.1.triggers_on_release = true;}
}

#[on_event]
impl OnEvent<TickEvent> for Button {
    fn on(&mut self, _ctx: &mut Context, event: TickEvent) -> Vec<Box<dyn DynEvent>> {
        self.2 = None;
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<MouseEvent> for Button {
    fn on(&mut self, _ctx: &mut Context, event: MouseEvent) -> Vec<Box<dyn DynEvent>> {
        match event.state {
            MouseState::Released(MouseButton::Left) if !self.1.1.is_pressed => {
                self.2 = Some(event.position);
            }
            MouseState::Pressed(MouseButton::Left) if event.position.is_some() => {
                if let Some(position) = self.2.take() {
                    let release = MouseEvent {position, state: MouseState::Released(MouseButton::Left)};
                    return prism::events![event, release];
                }
            }
            _ => {}
        }
        prism::events![event]
    }
}

impl std::ops::Deref for Button {
    type Target = _Button;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Button {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}


#[doc(hidden)]
#[derive(Component, Clone, Debug)]
pub struct _Button {
    layout: Stack,
    displays: Enum<Box<dyn Drawable>>,
    #[skip] disabled: bool,
    #[skip] on_click: Box<dyn Callback>,
    #[skip] disableable: bool,
    #[skip] triggers_on_release: bool,
    #[skip] is_pressed: bool,
    #[skip] active_label: Option<Instant>,
}
impl _Button {
    pub fn new(
        default: impl Drawable, hover: Option<impl Drawable>, pressed: Option<impl Drawable>,
        disabled: Option<impl Drawable>, feedback: Option<impl Drawable>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool, triggers_on_release: bool,
    ) -> Self {
        let mut items: Vec<(String, Box<dyn Drawable>)> = vec![
            ("default".to_string(), Box::new(default)),
        ];
        if let Some(hover) = hover {items.push(("hover".to_string(), Box::new(hover)));}
        if let Some(pressed) = pressed {items.push(("pressed".to_string(), Box::new(pressed)));}
        if let Some(disabled) = disabled {items.push(("disabled".to_string(), Box::new(disabled)));}
        if let Some(feedback) = feedback {items.push(("feedback".to_string(), Box::new(feedback)));}
        Self {
            layout: Stack::default(), displays: Enum::new(items, "default".to_string()),
            disabled: false, on_click: Box::new(callback), disableable, triggers_on_release,
            is_pressed: false, active_label: None,
        }
    }

    pub fn disable(&mut self, disable: bool) {
        if self.disabled != disable {
            self.disabled = disable;
            self.displays.display(if disable {"disabled"} else {"default"});
        }
    }

    pub fn on_click(&mut self) -> &mut Box<dyn Callback> {&mut self.on_click}

    fn callback(&mut self, ctx: &mut Context) {
        ctx.hardware.trigger_haptic();
        (self.on_click)(ctx);
    }

    fn display(&mut self, display: &str) {
        if self.displays.display("feedback") {
            self.active_label = Some(Instant::now());
        } else {
            self.displays.display(display);
        }
    }
}

#[on_event]
impl OnEvent<event::Button> for _Button {
    fn on(&mut self, ctx: &mut Context, event: event::Button) -> Vec<Box<dyn DynEvent>> {
        if let event::Button::Disable(disable) = event {
            if self.disableable {self.disable(disable);}
        } else if !self.disabled && self.active_label.is_none() {
            match event {
                event::Button::Hover(true) if !self.is_pressed => {self.displays.display("hover");}
                event::Button::Pressed(true) => {
                    self.is_pressed = true;
                    if !self.triggers_on_release {
                        self.callback(ctx);
                        self.display("pressed");
                    }
                }
                event::Button::Pressed(false) => {
                    let should_trigger = self.is_pressed && self.triggers_on_release;
                    self.is_pressed = false;
                    self.display("default");
                    if should_trigger {self.callback(ctx);}
                }
                event::Button::Hover(false) => {
                    self.is_pressed = false;
                    self.displays.display("default");
                }
                _ => {}
            }
        }
        vec![]
    }
}

#[on_event]
impl OnEvent<TickEvent> for _Button {
    fn on(&mut self, _ctx: &mut Context, event: TickEvent) -> Vec<Box<dyn DynEvent>> {
        if self.active_label.is_some_and(|timer| timer.elapsed() >= Duration::from_millis(1000)) {
            self.active_label = None;
            self.is_pressed = false;
            self.displays.display(if self.disabled {"disabled"} else {"default"});
        }
        prism::events![event]
    }
}
