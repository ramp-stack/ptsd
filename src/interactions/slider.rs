use prism::Context;
use prism::drawable::{Component, Drawable};
use prism::events::{DynEvent, OnEvent, on_event};

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use crate::display::Bin;
use crate::emitters;
use crate::events as event;
use crate::layout::{Offset, Padding, Size, Slider as SliderLayout, Stack};

type SliderClosure = Arc<Mutex<dyn FnMut(&mut Context, f32) + Send + Sync + 'static>>;

#[derive(Component, Clone, Debug)]
pub struct Slider(Stack, emitters::Slider<_Slider>);
impl Slider {
    pub fn new(
        start: f32, background: impl Drawable, foreground: impl Drawable, handle: impl Drawable,
        callback: impl FnMut(&mut Context, f32) + Send + Sync + 'static,
    ) -> Self {
        let slider = _Slider::new(start, background, foreground, handle, callback);
        Self(Stack::default(), emitters::Slider::new(slider))
    }
}

impl Deref for Slider {
    type Target = _Slider;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl DerefMut for Slider {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

/// The editable slider content exposed by `Slider` through `Deref`.
/// Cloned sliders share their callback; each clone retains its own value and visual children.
#[derive(Component, Clone)]
pub struct _Slider {
    layout: SliderLayout,
    pub background: Bin<Stack, Box<dyn Drawable>>,
    pub foreground: Bin<Stack, Box<dyn Drawable>>,
    pub handle: Bin<Stack, Box<dyn Drawable>>,
    #[skip] closure: SliderClosure,
}
impl _Slider {
    pub fn new(
        start: f32, background: impl Drawable, foreground: impl Drawable, mut handle: impl Drawable,
        callback: impl FnMut(&mut Context, f32) + Send + Sync + 'static,
    ) -> Self {
        let min = handle.request_width().0;
        let width = Size::custom(move |widths: Vec<(f32, f32)>| (widths[0].0.min(min), f32::MAX));
        let b_layout = Stack::new(
            Offset::Start, Offset::Center, width, Size::Static(6.0), Padding::default(),
        );
        let f_layout = Stack::new(
            Offset::Start, Offset::Start, Size::custom(|_| (0.0, f32::MAX)),
            Size::Static(6.0), Padding::default(),
        );
        let k_layout = Stack::new(
            Offset::Start, Offset::Start, Size::Fill, Size::Fit, Padding::default(),
        );
        Self {
            layout: SliderLayout::new(start, Size::Fit),
            background: Bin(b_layout, Box::new(background)),
            foreground: Bin(f_layout, Box::new(foreground)), handle: Bin(k_layout, Box::new(handle)),
            closure: Arc::new(Mutex::new(callback)),
        }
    }
}

impl Deref for _Slider {
    type Target = SliderLayout;
    fn deref(&self) -> &Self::Target {&self.layout}
}

impl DerefMut for _Slider {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.layout}
}

impl std::fmt::Debug for _Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {f.write_str("_Slider")}
}

#[on_event]
impl OnEvent<event::Slider> for _Slider {
    fn on(&mut self, ctx: &mut Context, event: event::Slider) -> Vec<Box<dyn DynEvent>> {
        let (event::Slider::Start(x) | event::Slider::Moved(x)) = event;
        self.value = (self.layout.position(x) * 100.0).round() / 100.0;
        if matches!(event, event::Slider::Start(_)) {ctx.hardware.trigger_haptic();}
        (self.closure.lock().unwrap())(ctx, self.value);
        prism::events![event]
    }
}
