use prism::event::{self, OnEvent, Event, TickEvent};
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
        let button = _Button::new(default, hover, pressed, disabled, callback, disableable);
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
pub struct _Button(Stack, Enum<Box<dyn Drawable>>, #[skip] bool, #[skip] Box<dyn Callback>, #[skip] bool);

impl _Button {
    pub fn new(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool,
    ) -> Self {
        let mut items: Vec<(String, Box<dyn Drawable>)> = Vec::new();
        items.push(("default".to_string(), Box::new(default)));
        if let Some(h) = hover { items.push(("hover".to_string(), Box::new(h))) }
        if let Some(p) = pressed { items.push(("pressed".to_string(), Box::new(p))) }
        if let Some(d) = disabled { items.push(("disabled".to_string(), Box::new(d))) }
        _Button(Stack::default(), Enum::new(items, "default".to_string()), false, Box::new(callback), disableable)
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
}

impl OnEvent for _Button {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if self.4 && let Some(event::Button::Disable(disable)) = event.downcast_ref::<event::Button>() {
            self.disable(*disable);
        } else if let Some(event) = event.downcast_ref::<event::Button>() && !self.2 {
            match event {
                event::Button::Hover(true) => self.1.display("hover"),
                event::Button::Pressed(true) => {
                    ctx.send(Request::Hardware(Hardware::Haptic));
                    self.1.display("pressed");
                    (self.3)(ctx);
                }
                event::Button::Pressed(false) => self.1.display("default"),
                _ => self.1.display("default"),
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

