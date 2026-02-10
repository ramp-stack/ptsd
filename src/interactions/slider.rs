use prism::event::{self, OnEvent, Event, TickEvent};
use prism::drawable::{Drawable, Component, SizedTree};
use prism::display::Bin;
use prism::layout::{Stack, Size, Offset, Padding};
use prism::{emitters, Context, Request, Hardware};

#[derive(Component, Debug)]
pub struct Slider(Stack, emitters::Slider<_Slider>);
impl OnEvent for Slider {}
impl Slider {
    pub fn new(
        start: f32, 
        background: impl Drawable + 'static,
        foreground: impl Drawable + 'static,
        handle: impl Drawable + 'static,
        callback: impl FnMut(&mut Context, f32) + 'static
    ) -> Self {
        let slider = _Slider::new(start, background, foreground, handle, callback);
        Self(Stack::default(), emitters::Slider::new(slider))
    }
}

impl std::ops::Deref for Slider {
    type Target = _Slider;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Slider {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}

#[derive(Component)]
pub struct _Slider {
    layout: Stack,
    pub background: Bin<Stack, Box<dyn Drawable>>,
    pub foreground: Bin<Stack, Box<dyn Drawable>>,
    pub handle: Bin<Stack, Box<dyn Drawable>>,
    #[skip] pub value: f32,
    #[skip] closure: SliderClosure,
}

impl _Slider {
    pub fn new(
        start: f32, 
        background: impl Drawable + 'static,
        foreground: impl Drawable + 'static,
        handle: impl Drawable + 'static,
        callback: impl FnMut(&mut Context, f32) + 'static
    ) -> Self {
        let min = Drawable::request_size(&handle).0.min_width();
        let width = Size::custom(move |widths: Vec<(f32, f32)>| (widths[0].0.min(min), f32::MAX));
        let b_layout = Stack(Offset::Start, Offset::Center, width, Size::Static(6.0), Padding::default());
        let f_layout = Stack(Offset::Start, Offset::Start, Size::Static(30.0), Size::Static(6.0), Padding::default());
        let k_layout = Stack(Offset::Start, Offset::Start, Size::Fit, Size::Fit, Padding::default());
        let layout = Stack(Offset::Start, Offset::Center, Size::Fit, Size::Fit, Padding::default());

        _Slider {
            layout,
            background: Bin(b_layout, Box::new(background)),
            foreground: Bin(f_layout, Box::new(foreground)),
            handle: Bin(k_layout, Box::new(handle)),
            value: start, 
            closure: Box::new(callback),
        }
    }
}

impl OnEvent for _Slider {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_ref::<event::Slider>() {
            (self.closure)(ctx, self.value);
            match event {
                event::Slider::Moved(x) => self.value = ((*x / sized.0.0) * 100.0).round() / 100.0,
                event::Slider::Start(x) => {
                    self.value = ((*x / sized.0.0) * 100.0).round() / 100.0;
                    ctx.send(Request::Hardware(Hardware::Haptic));
                },
            }
        } else if event.downcast_ref::<TickEvent>().is_some() {
            let handle_size = Drawable::request_size(&(**self.handle.inner())).0.min_width() / 2.0;
            let clamped_x = (sized.0.0 * self.value).clamp(0.0, sized.0.0);
            self.handle.get_layout().0 = Offset::Static((clamped_x - handle_size).max(0.0));
            self.foreground.get_layout().2 = Size::Static(clamped_x);
        }

        vec![event]
    }
}

impl std::fmt::Debug for _Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_Slider")
    }
}

type SliderClosure = Box<dyn FnMut(&mut Context, f32)>;