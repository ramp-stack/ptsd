use prism::event::{self, OnEvent, Event};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::display::Enum;
use prism::layout::Stack;
use prism::{emitters, Context, Request, Hardware};

#[derive(Component, Debug)]
pub struct Selectable(Stack, emitters::Selectable<_Selectable>);
impl OnEvent for Selectable {}
impl Selectable {
    pub fn new(
        default: impl Drawable + 'static,
        selected: impl Drawable + 'static,
        is_selected: bool,
        can_deselect: bool,
        on_click: impl FnMut(&mut Context) + 'static,
        group_id: uuid::Uuid,
    ) -> Self {
        let selectable = _Selectable::new(default, selected, is_selected, can_deselect, on_click);
        Self(Stack::default(), emitters::Selectable::new(selectable, group_id))
    }
}

impl std::ops::Deref for Selectable {
    type Target = _Selectable;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Selectable {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component)]
pub struct _Selectable(Stack, Enum, #[skip] Box<dyn FnMut(&mut Context)>, #[skip] bool);

impl _Selectable {
    pub fn new(
        default: impl Drawable + 'static,
        selected: impl Drawable + 'static,
        is_selected: bool,
        can_deselect: bool,
        on_click: impl FnMut(&mut Context) + 'static
    ) -> Self {
        let start = if is_selected {"selected"} else {"default"};
        _Selectable(Stack::default(), Enum::new(vec![
            ("default".to_string(), Box::new(default)),
            ("selected".to_string(), Box::new(selected)),
        ], start.to_string()), Box::new(on_click), can_deselect)
    }
}

impl OnEvent for _Selectable {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event::Selectable::Selected(b)) = event.downcast_ref::<event::Selectable>() {
            match b {
                false => self.1.display("default"),
                true => {
                    if self.3 && &self.1.current() == "selected" {
                        // already selected 
                        self.1.display("default");
                        ctx.send(Request::Hardware(Hardware::Haptic));
                        (self.2)(ctx);
                    } else {
                        self.1.display("selected");
                        ctx.send(Request::Hardware(Hardware::Haptic));
                        (self.2)(ctx);
                    }
                }
            }
        }
        vec![event]
    }
}

impl std::fmt::Debug for _Selectable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Selectable")
    }
}
