use prism::event::{self, OnEvent, Event};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::display::Enum;
use prism::layout::Stack;
use prism::{emitters, Context, Request, Hardware};

use std::sync::{Arc, Mutex};

#[derive(Component, Clone, Debug)]
pub struct Toggle(Stack, emitters::Button<_Toggle>);
impl OnEvent for Toggle {}
impl Toggle {
    pub fn new(
        on: impl Drawable + 'static,
        off: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context, bool) + Send + Sync + 'static,
    ) -> Self {
        let toggle = _Toggle::new(on, off, is_selected, on_click);
        Self(Stack::default(), emitters::Button::new(toggle))
    }
}

impl std::ops::Deref for Toggle {
    type Target = _Toggle;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Toggle {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component, Clone)]
pub struct _Toggle(Stack, Enum<Box<dyn Drawable>>, #[skip] bool, #[skip] ToggleCallback);

impl _Toggle {
    pub fn new(
        on: impl Drawable + 'static,
        off: impl Drawable + 'static,
        is_selected: bool,
        on_click: impl FnMut(&mut Context, bool) + Send + Sync + 'static,
    ) -> Self {
        let start = if is_selected {"on"} else {"off"};
        _Toggle(Stack::default(), 
            Enum::new(vec![("on".to_string(), Box::new(on)), ("off".to_string(), Box::new(off))], start.to_string()), 
            !is_selected, Arc::new(Mutex::new(on_click))
        )
    }
}

impl OnEvent for _Toggle {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event::Button::Pressed(true)) = event.downcast_ref::<event::Button>() {
            self.2 = !self.2;
            ctx.send(Request::Hardware(Hardware::Haptic));
            if let Ok(mut cb) = self.3.lock() { (cb)(ctx, !self.2); }
            match self.2 {
                false => self.1.display("on"),
                true => self.1.display("off"),
            }
        }
        Vec::new()
    }
}

impl std::fmt::Debug for _Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Toggle")
    }
}


type ToggleCallback = Arc<Mutex<dyn FnMut(&mut Context, bool) + Send + Sync + 'static>>;