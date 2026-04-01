use prism::event::{self, OnEvent, Event};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::display::Enum;
use prism::layout::Stack;
use prism::{emitters, Context, Request, Hardware};

use crate::utils::Callback;

#[derive(Component, Debug, Clone)]
pub struct Button(Stack, emitters::Button<_Button>);
impl OnEvent for Button {}
impl Button {
    pub fn new(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool,
    ) -> Self {
        let button = _Button::new(default, hover, pressed, disabled, callback, disableable, false);
        Self(Stack::default(), emitters::Button::new(button))
    }

    pub fn new_triggers_on_release(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool,
    ) -> Self {
        let button = _Button::new(default, hover, pressed, disabled, callback, disableable, true);
        Self(Stack::default(), emitters::Button::new(button))
    }
}

impl std::ops::Deref for Button {
    type Target = _Button;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Button {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component, Clone)]
pub struct _Button(Stack, Enum<Box<dyn Drawable>>, #[skip] bool, #[skip] Box<dyn Callback>, #[skip] bool, #[skip] bool, #[skip] bool);

impl _Button {
    pub fn new(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool,
        triggers_on_release: bool,
    ) -> Self {
        let mut items: Vec<(String, Box<dyn Drawable>)> = Vec::new();
        items.push(("default".to_string(), Box::new(default)));
        if let Some(h) = hover { items.push(("hover".to_string(), Box::new(h))) }
        if let Some(p) = pressed { items.push(("pressed".to_string(), Box::new(p))) }
        if let Some(d) = disabled { items.push(("disabled".to_string(), Box::new(d))) }
        _Button(Stack::default(), Enum::new(items, "default".to_string()), false, Box::new(callback), disableable, triggers_on_release, false)
    }

    pub fn disable(&mut self, disable: bool) {
        if self.2 != disable {
            self.2 = disable;

            match self.2 {
                true => self.1.display("disabled"),
                false => self.1.display("default")
            }
        }
    }

    pub fn on_click(&mut self) -> &mut Box<dyn Callback> {&mut self.3}
}

impl OnEvent for _Button {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<event::Button>() {
            if let event::Button::Disable(disable) = event {
                if self.4 { self.disable(*disable);} 
            } else if !self.2 {
                match event {
                    event::Button::Hover(true) if !self.6 => self.1.display("hover"),
                    event::Button::Pressed(true) => {
                        self.6 = true;
                        self.1.display("pressed");
                        if !self.5 {
                            ctx.send(Request::Hardware(Hardware::Haptic));
                            (self.3)(ctx);
                        }
                    }
                    event::Button::Pressed(false) => {
                        self.6 = false;
                        if self.5 {
                            ctx.send(Request::Hardware(Hardware::Haptic));
                            (self.3)(ctx);
                        }
                        self.1.display("default");
                    },
                    event::Button::Hover(false) if !self.6 => {
                        self.1.display("default");
                    }
                    // event::Button::Disable(_) => {},
                    _ => {} //self.1.display("default"),
                }
            }
        }

        vec![event]
    }
}

impl std::fmt::Debug for _Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Button")
    }
}

