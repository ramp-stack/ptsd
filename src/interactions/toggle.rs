use std::sync::{Arc, Mutex};

use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::DynEvent;

use crate::display::Enum;
use crate::emitters;
use crate::events as event;
use crate::layout::Stack;

#[derive(Component, Clone, Debug)]
pub struct Toggle(Stack, emitters::Button<_Toggle>);
impl Toggle {
    pub fn new(
        on: impl Drawable, off: impl Drawable, is_selected: bool,
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


type ToggleCallback = Arc<Mutex<dyn FnMut(&mut Context, bool) + Send + Sync + 'static>>;

#[doc(hidden)]
#[derive(Component, Clone)]
#[on_dyn_event]
pub struct _Toggle(Stack, Enum<Box<dyn Drawable>>, #[skip] bool, #[skip] ToggleCallback);
impl _Toggle {
    pub fn new(
        on: impl Drawable, off: impl Drawable, is_selected: bool,
        on_click: impl FnMut(&mut Context, bool) + Send + Sync + 'static,
    ) -> Self {
        let start = if is_selected {"on"} else {"off"};
        Self(Stack::default(), Enum::new(vec![
            ("on".to_string(), Box::new(on)), ("off".to_string(), Box::new(off)),
        ], start.to_string()), !is_selected, Arc::new(Mutex::new(on_click)))
    }

    fn on_event(&mut self, ctx: &mut Context, incoming: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        if let Some(event::Button::Pressed(true)) = incoming.downcast_ref() {
            self.2 = !self.2;
            ctx.hardware.trigger_haptic();
            if let Ok(mut callback) = self.3.lock() {callback(ctx, !self.2);}
            self.1.display(if self.2 {"off"} else {"on"});
        }
        vec![]
    }
}

impl std::fmt::Debug for _Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {f.write_str("_Toggle")}
}
