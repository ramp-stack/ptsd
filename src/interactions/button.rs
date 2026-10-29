use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{on_change, on_event, DynEvent, OnEvent, OnChange, Timer};

use std::time::Duration;

use crate::display::Enum;
use crate::emitters;
use crate::events as event;
use crate::layout::Stack;
use crate::utils::Callback;

#[derive(Component, Debug, Clone)]
pub struct Button(Stack, emitters::Button<_Button>);
impl Button {
    pub fn new(
        default: impl Drawable, hover: Option<impl Drawable>, pressed: Option<impl Drawable>,
        disabled: Option<impl Drawable>, feedback: Option<impl Drawable>,
        callback: impl FnMut(&mut Context) + Clone + 'static, disableable: bool,
    ) -> Self {
        let button = _Button::new(
            default, hover, pressed, disabled, feedback, callback, disableable,
        );
        Self(Stack::default(), emitters::Button::new(button))
    }

    pub fn new_triggers_on_release(
        default: impl Drawable, hover: Option<impl Drawable>, pressed: Option<impl Drawable>,
        disabled: Option<impl Drawable>, feedback: Option<impl Drawable>,
        callback: impl FnMut(&mut Context) + Clone + 'static, disableable: bool,
    ) -> Self {
        let mut button = Self::new(default, hover, pressed, disabled, feedback, callback, disableable);
        button.set_trigger_on_release();
        button
    }

    pub fn set_trigger_on_release(&mut self) {self.1.1.triggers_on_release = true;}
}

impl std::ops::Deref for Button {
    type Target = _Button;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Button {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

/// Button state exposed through `Button`'s `Deref` implementation for its state-changing methods.
#[derive(Component, Clone, Debug)]
pub struct _Button {
    layout: Stack,
    displays: Enum<Box<dyn Drawable>>,
    #[skip] disabled: bool,
    #[skip] on_click: Box<dyn Callback>,
    #[skip] disableable: bool,
    #[skip] triggers_on_release: bool,
    #[skip] is_pressed: bool,
    #[skip] feedback: Timer,
}
impl _Button {
    pub fn new(
        default: impl Drawable, hover: Option<impl Drawable>, pressed: Option<impl Drawable>,
        disabled: Option<impl Drawable>, feedback: Option<impl Drawable>,
        callback: impl FnMut(&mut Context) + Clone + 'static, disableable: bool,
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
            disabled: false, on_click: Box::new(callback), disableable, triggers_on_release: false,
            is_pressed: false, feedback: Timer::default(),
        }
    }

    pub fn disable(&mut self, disable: bool) {
        if self.disabled != disable {
            self.disabled = disable;
            self.feedback.cancel();
            self.is_pressed = false;
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
            self.feedback.reset(Duration::from_secs(1));
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
        } else if let event::Button::Hover(false) = event {
            self.is_pressed = false;
            if !self.disabled && !self.feedback.is_active() {self.displays.display("default");}
        } else if !self.disabled && !self.feedback.is_active() {
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
                _ => {}
            }
        }
        vec![]
    }
}

#[on_change(self.feedback.next())]
impl OnChange<()> for _Button {
    fn on_change(&mut self, _ctx: &mut Context, _: ()) {
        self.is_pressed = false;
        self.displays.display(if self.disabled {"disabled"} else {"default"});
    }
}
