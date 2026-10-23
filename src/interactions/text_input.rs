use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{on_event, DynEvent, OnEvent};

use crate::display::Enum;
use crate::emitters;
use crate::events as event;
use crate::interface::interfaces::ShowKeyboard;
use crate::layout::{Offset, Padding, Size, Stack};

#[derive(Component, Clone, Debug)]
pub struct InputField(Stack, emitters::Selectable<emitters::TextInput<_InputField>>);
impl InputField {
    pub fn new(
        default: impl Drawable, focus: impl Drawable, hover: Option<impl Drawable>,
        error: Option<impl Drawable>, content: impl Drawable, height: f32,
    ) -> Self {
        let input = _InputField::new(default, focus, hover, error, content, height);
        Self(Stack::default(), emitters::TextInput::new(input, true))
    }

    pub fn is_focused(&self) -> bool {self.1.1.1.1.current() == "focus"}
    pub fn focus(&self, ctx: &mut Context) {self.1.select(ctx);}
}

impl std::ops::Deref for InputField {
    type Target = _InputField;
    fn deref(&self) -> &Self::Target {&self.1.1.1}
}

impl std::ops::DerefMut for InputField {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1.1}
}

/// Input content exposed through `InputField`'s `Deref` implementation.
/// Call `error` to change validation state together with its displayed content.
#[derive(Debug, Component, Clone)]
pub struct _InputField(Stack, Enum<Box<dyn Drawable>>, pub Box<dyn Drawable>, #[skip] bool);
impl _InputField {
    pub fn new(
        default: impl Drawable, focus: impl Drawable, hover: Option<impl Drawable>,
        error: Option<impl Drawable>, content: impl Drawable, height: f32,
    ) -> Self {
        let height = Size::custom(move |h: Vec<(f32, f32)>| (h[1].0.max(height), h[1].1.max(height)));
        let layout = Stack(Offset::Start, Offset::Start, Size::Fit, height, Padding::default());
        let mut items: Vec<(String, Box<dyn Drawable>)> = vec![
            ("default".to_string(), Box::new(default)), ("focus".to_string(), Box::new(focus)),
        ];
        if let Some(hover) = hover {items.push(("hover".to_string(), Box::new(hover)));}
        if let Some(error) = error {items.push(("error".to_string(), Box::new(error)));}
        Self(layout, Enum::new(items, "default".to_string()), Box::new(content), false)
    }

    pub fn error(&mut self, error: bool) {
        if self.3 != error {
            self.3 = error;
            self.1.display(if error {"error"} else {"default"});
        }
    }
}

#[on_event]
impl OnEvent<event::TextInput> for _InputField {
    fn on(&mut self, ctx: &mut Context, event: event::TextInput) -> Vec<Box<dyn DynEvent>> {
        match &event {
            event::TextInput::Hover(true) if !self.3 => {self.1.display("hover");}
            event::TextInput::Focused(true) => {
                ctx.emit(ShowKeyboard(true));
                ctx.hardware.trigger_haptic();
                self.1.display("focus");
            }
            event::TextInput::Focused(false) => {
                self.1.display(if self.3 {"error"} else {"default"});
            }
            _ => {self.1.display("default");}
        }
        prism::events![event]
    }
}
