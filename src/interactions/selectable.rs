use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{on_event, DynEvent, OnEvent};

use crate::display::Enum;
use crate::emitters;
use crate::events as event;
use crate::layout::Stack;
use crate::utils::Callback;

#[derive(Component, Debug, Clone)]
pub struct Selectable(Stack, emitters::Selectable<_Selectable>);
impl Selectable {
    pub fn new(
        default: impl Drawable, selected: impl Drawable, is_selected: bool, can_deselect: bool,
        on_click: impl FnMut(&mut Context) + Clone + 'static, group_id: uuid::Uuid,
    ) -> Self {
        let selectable = _Selectable::new(default, selected, is_selected, can_deselect, on_click);
        Self(Stack::default(), emitters::Selectable::new(selectable, group_id))
    }

    pub fn is_selected(&self) -> bool {self.1.1.4}
}

impl std::ops::Deref for Selectable {
    type Target = _Selectable;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Selectable {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}


#[doc(hidden)]
#[derive(Component, Clone, Debug)]
pub struct _Selectable(
    Stack, Enum<Box<dyn Drawable>>, #[skip] Box<dyn Callback>, #[skip] bool, #[skip] bool,
);
impl _Selectable {
    pub fn new(
        default: impl Drawable, selected: impl Drawable, is_selected: bool, can_deselect: bool,
        on_click: impl FnMut(&mut Context) + Clone + 'static,
    ) -> Self {
        let start = if is_selected {"selected"} else {"default"};
        Self(Stack::default(), Enum::new(vec![
            ("default".to_string(), Box::new(default)),
            ("selected".to_string(), Box::new(selected)),
        ], start.to_string()), Box::new(on_click), can_deselect, is_selected)
    }
}

#[on_event]
impl OnEvent<event::Selectable> for _Selectable {
    fn on(&mut self, ctx: &mut Context, event: event::Selectable) -> Vec<Box<dyn DynEvent>> {
        if let event::Selectable::Selected(selected) = &event {
            if !selected {
                self.1.display("default");
                self.4 = false;
            } else {
                self.4 = !(self.3 && self.1.current() == "selected");
                self.1.display(if self.4 {"selected"} else {"default"});
                ctx.hardware.trigger_haptic();
                (self.2)(ctx);
            }
        }
        prism::events![event]
    }
}
