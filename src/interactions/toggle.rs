use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{DynEvent, OnEvent, on_event};

use std::sync::{Arc, Mutex};

use crate::display::Enum;
use crate::emitters;
use crate::events as event;
use crate::layout::Stack;

type ToggleCallback = Arc<Mutex<dyn FnMut(&mut Context, bool) + Send + Sync + 'static>>;

#[derive(Component, Clone, Debug)]
pub struct Toggle(Stack, emitters::Button<_Toggle>);
impl Toggle {
    pub fn new(on: impl Drawable, off: impl Drawable, is_selected: bool, on_click: impl FnMut(&mut Context, bool) + Send + Sync + 'static) -> Self {
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

/// The toggle content exposed by `Toggle` through `Deref`.
/// Cloned toggles share their callback while retaining independent selection state.
/// Button events are consumed; other event types continue to the displayed content.
#[derive(Component, Clone)]
pub struct _Toggle(Stack, Enum<Box<dyn Drawable>>, #[skip] bool, #[skip] ToggleCallback);
impl _Toggle {
    pub fn new(on: impl Drawable, off: impl Drawable, is_selected: bool, on_click: impl FnMut(&mut Context, bool) + Send + Sync + 'static) -> Self {
        let start = if is_selected {"on"} else {"off"};
        Self(Stack::default(), Enum::new(vec![
            ("on".to_string(), Box::new(on)), ("off".to_string(), Box::new(off)),
        ], start.to_string()), !is_selected, Arc::new(Mutex::new(on_click)))
    }
}

#[on_event]
impl OnEvent<event::Button> for _Toggle {
    fn on(&mut self, ctx: &mut Context, event: event::Button) -> Vec<Box<dyn DynEvent>> {
        if let event::Button::Pressed(true) = event {
            self.2 = !self.2;
            ctx.hardware.trigger_haptic();
            (self.3.lock().unwrap())(ctx, !self.2);
            self.1.display(if self.2 {"off"} else {"on"});
        }
        vec![]
    }
}

impl std::fmt::Debug for _Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {f.write_str("_Toggle")}
}
