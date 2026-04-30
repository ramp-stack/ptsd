use prism::event::{self, OnEvent, Event};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::display::Enum;
use prism::layout::{Stack, Size, Offset, Padding};
use prism::{emitters, Context};
use crate::interfaces::ShowKeyboard;
// use crate::components::interface::ShowKeyboard;

#[derive(Component, Clone, Debug)]
pub struct InputField(Stack, emitters::Selectable<emitters::TextInput<_InputField>>);
impl OnEvent for InputField {}
impl InputField {
    pub fn new(
        default: impl Drawable + 'static,
        focus: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        error: Option<impl Drawable + 'static>,
        content: impl Drawable + 'static,
        height: f32,
    ) -> Self {
        let text_input = _InputField::new(default, focus, hover, error, content, height);
        Self(Stack::default(), emitters::TextInput::new(text_input, true))
    }
}

impl std::ops::Deref for InputField {
    type Target = _InputField;
    fn deref(&self) -> &Self::Target {&self.1.1.1}
}

impl std::ops::DerefMut for InputField {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1.1}
}

#[derive(Debug, Component, Clone)]
pub struct _InputField(Stack, Enum<Box<dyn Drawable>>, pub Box<dyn Drawable>, #[skip] pub bool);

impl _InputField {
    pub fn new(
        default: impl Drawable + 'static,
        focus: impl Drawable + 'static,
        hover: Option<impl Drawable + 'static>,
        error: Option<impl Drawable + 'static>,
        content: impl Drawable + 'static,
        height: f32,
    ) -> Self {
        let height = Size::custom(move |h: Vec<(f32, f32)>| (h[1].0.max(height), h[1].1.max(height)));
        let layout = Stack(Offset::Start, Offset::Start, Size::Fit, height, Padding::default());

        let mut items: Vec<(String, Box<dyn Drawable>)> = Vec::new();
        items.push(("default".to_string(), Box::new(default)));
        items.push(("focus".to_string(), Box::new(focus)));
        if let Some(h) = hover { items.push(("hover".to_string(), Box::new(h))) }
        if let Some(e) = error { items.push(("error".to_string(), Box::new(e))) }

        _InputField(layout, Enum::new(items, "default".to_string()), Box::new(content), false)
    }

    pub fn error(&mut self, error: bool) {
        if self.3 != error {
            self.3 = error;

            match self.3 {
                true => self.1.display("error"),
                false => self.1.display("default")
            }
        }
    }
}

impl OnEvent for _InputField {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(e) = event.downcast_ref::<event::TextInput>() {
            match e {
                event::TextInput::Hover(true) if !self.3 => self.1.display("hover"),
                event::TextInput::Focused(true) => {
                    ctx.emit(ShowKeyboard(true));
                    ctx.trigger_haptic();
                    self.1.display("focus");
                },
                event::TextInput::Focused(false) => {
                    // ctx.trigger_event(ShowKeyboard(false));
                    self.1.display(if self.3 {"error"} else {"default"});
                },
                _ => self.1.display("default"),
            }
        }
        
        vec![event]
    }
}