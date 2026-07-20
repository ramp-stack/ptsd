use prism::event::{self, OnEvent, Event, TickEvent};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::display::Enum;
use prism::layout::Stack;
use prism::{emitters, Context};

use std::time::{Instant, Duration};

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
        feedback: Option<impl Drawable + 'static>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool,
    ) -> Self {
        let button = _Button::new(default, hover, pressed, disabled, feedback, callback, disableable, false);
        Self(Stack::default(), emitters::Button::new(button))
    }

    pub fn new_triggers_on_release(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        feedback: Option<impl Drawable + 'static>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool,
    ) -> Self {
        let button = _Button::new(default, hover, pressed, disabled, feedback, callback, disableable, true);
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
pub struct _Button {
    layout: Stack,
    displays: Enum<Box<dyn Drawable>>,
    #[skip] disabled: bool,
    #[skip] on_click: Box<dyn Callback>,
    #[skip] disableable: bool,
    #[skip] triggers_on_release: bool,
    #[skip] is_pressed: bool,
    #[skip] active_label: Option<(Instant, String)>
}

impl _Button {
    pub fn new(
        default: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        pressed: Option<impl Drawable + 'static>,
        disabled: Option<impl Drawable + 'static>,
        feedback: Option<impl Drawable + 'static>,
        callback: impl FnMut(&mut Context) + Clone + 'static,
        disableable: bool,
        triggers_on_release: bool,
    ) -> Self {
        let mut items: Vec<(String, Box<dyn Drawable>)> = Vec::new();
        items.push(("default".to_string(), Box::new(default)));
        if let Some(h) = hover { items.push(("hover".to_string(), Box::new(h))) }
        if let Some(p) = pressed { items.push(("pressed".to_string(), Box::new(p))) }
        if let Some(d) = disabled { items.push(("disabled".to_string(), Box::new(d))) }
        if let Some(d) = feedback { items.push(("feedback".to_string(), Box::new(d))) }

        _Button {
            layout: Stack::default(),
            displays: Enum::new(items, "default".to_string()),
            disabled: false,
            on_click: Box::new(callback),
            disableable,
            triggers_on_release,
            is_pressed: false,
            active_label: None,
        }
    }

    pub fn disable(&mut self, disable: bool) {
        if self.disabled != disable {
            self.disabled = disable;

            match self.disabled {
                true => self.displays.display("disabled"),
                false => self.displays.display("default")
            };
        }
    }

    pub fn on_click(&mut self) -> &mut Box<dyn Callback> {&mut self.on_click}

    fn callback(&mut self, ctx: &mut Context) {
        ctx.trigger_haptic();
        (self.on_click)(ctx);
    }

    fn display(&mut self, display: &str) {
        if self.displays.display("feedback") {
            self.active_label = Some((Instant::now(), display.to_string()))
        } else {
            let _ = self.displays.display(display);
        }
    }

    fn handle_button_event(&mut self, ctx: &mut Context, event: event::Button) {
        if let event::Button::Disable(disable) = event {
            if self.disableable { self.disable(disable);} 
        } else if !self.disabled && self.active_label.is_none() {
            match event {
                event::Button::Hover(true) if !self.is_pressed => {self.displays.display("hover");},
                event::Button::Pressed(true) => {
                    self.is_pressed = true;
                    if !self.triggers_on_release {
                        self.callback(ctx);
                        self.display("default");
                    }
                }
                event::Button::Pressed(false) => {
                    self.is_pressed = false;
                    if self.triggers_on_release {
                        self.callback(ctx);
                        self.display("default");
                    } else {
                        self.displays.display("default");
                    }
                },
                event::Button::Hover(false) if !self.is_pressed => {
                    self.displays.display("default");
                }
                // event::Button::Disable(_) => {},
                _ => {} //self.1.display("default"),
            }
        }
    }
}

impl OnEvent for _Button {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<TickEvent>().is_some() {
            if let Some((timer, display)) = &self.active_label {
                if timer.elapsed() >= Duration::from_millis(1000) {
                    self.active_label = None;
                    self.handle_button_event(ctx, event::Button::Pressed(false));
                }
            }
        } else if let Some(event) = event.downcast_ref::<event::Button>() {
            self.handle_button_event(ctx, *event);
            return vec![];
        }

        vec![event]
    }
}

impl std::fmt::Debug for _Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Button")
    }
}

