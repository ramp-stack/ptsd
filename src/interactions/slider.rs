use std::sync::{Arc, Mutex};

use prism::Context;
use prism::drawable::{Area, Component, Drawable, Instruction, Layout, LayoutTree, RangeTree, Rect};
use prism::events::{ChildEvent, DynEvent, TickEvent};

use crate::display::Bin;
use crate::emitters;
use crate::events as event;
use crate::layout::{Offset, Padding, Size, Stack};

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

impl std::ops::Deref for Slider {
    type Target = _Slider;
    fn deref(&self) -> &Self::Target {&self.1.1}
}

impl std::ops::DerefMut for Slider {
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.1.1}
}


type SliderClosure = Arc<Mutex<dyn FnMut(&mut Context, f32) + Send + Sync + 'static>>;

#[doc(hidden)]
#[derive(Clone)]
pub struct _Slider {
    layout: Stack,
    pub background: Bin<Stack, Box<dyn Drawable>>,
    pub foreground: Bin<Stack, Box<dyn Drawable>>,
    pub handle: Bin<Stack, Box<dyn Drawable>>,
    pub value: f32,
    closure: SliderClosure,
}
impl _Slider {
    pub fn new(
        start: f32, background: impl Drawable, foreground: impl Drawable, handle: impl Drawable,
        callback: impl FnMut(&mut Context, f32) + Send + Sync + 'static,
    ) -> Self {
        let min = handle.size_range().0.min_width;
        let width = Size::custom(move |widths: Vec<(f32, f32)>| (widths[0].0.min(min), f32::MAX));
        let b_layout = Stack(
            Offset::Start, Offset::Center, width, Size::Static(6.0), Padding::default(),
        );
        let f_layout = Stack(
            Offset::Start, Offset::Start, Size::Static(30.0), Size::Static(6.0), Padding::default(),
        );
        let k_layout = Stack(Offset::Start, Offset::Start, Size::Fill, Size::Fit, Padding::default());
        let layout = Stack(Offset::Start, Offset::Center, Size::Fit, Size::Fit, Padding::default());
        Self {
            layout, background: Bin(b_layout, Box::new(background)),
            foreground: Bin(f_layout, Box::new(foreground)), handle: Bin(k_layout, Box::new(handle)),
            value: start, closure: Arc::new(Mutex::new(callback)),
        }
    }
}

impl Drawable for _Slider {
    fn size_range(&self) -> RangeTree {
        let requests = vec![
            self.background.size_range(), self.foreground.size_range(), self.handle.size_range(),
        ];
        let range = self.layout.size_range(requests.iter().map(|request| request.0).collect());
        RangeTree(range, requests)
    }

    fn layout(&mut self, size: (f32, f32), request: &RangeTree) -> LayoutTree {
        let size = request.0.clamp(size);
        let areas = self.layout.layout(size, request.1.iter().map(|request| request.0).collect());
        let children: [&mut dyn Drawable; 3] = [
            &mut self.background, &mut self.foreground, &mut self.handle,
        ];
        LayoutTree(size, areas.into_iter().zip(children).zip(&request.1)
            .map(|((area, child), request)| (area.offset, child.layout(area.size, request)))
            .collect())
    }

    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let children: [&dyn Drawable; 3] = [&self.background, &self.foreground, &self.handle];
        layout.1.iter().zip(children).flat_map(|((child_offset, branch), child)| {
            let offset = (offset.0 + child_offset.0, offset.1 + child_offset.1);
            let left = bound.0.max(offset.0);
            let top = bound.1.max(offset.1);
            let bound = (
                left, top, (bound.0 + bound.2).min(offset.0 + branch.0.0) - left,
                (bound.1 + bound.3).min(offset.1 + branch.0.1) - top,
            );
            if bound.2 > 0.0 && bound.3 > 0.0 {child.draw(branch, offset, bound)} else {vec![]}
        }).collect()
    }

    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, incoming: Box<dyn DynEvent>) {
        if let Some(slider) = incoming.downcast_ref::<event::Slider>() {
            match slider {
                event::Slider::Moved(x) => self.value = ((*x / layout.0.0) * 100.0).round() / 100.0,
                event::Slider::Start(x) => {
                    self.value = ((*x / layout.0.0) * 100.0).round() / 100.0;
                    ctx.hardware.trigger_haptic();
                }
            }
            if let Ok(mut callback) = self.closure.lock() {callback(ctx, self.value);}
        } else if incoming.downcast_ref::<TickEvent>().is_some() {
            let handle_size = self.handle.inner().size_range().0.min_width / 2.0;
            let clamped_x = (layout.0.0 * self.value).clamp(0.0, layout.0.0);
            let end = (layout.0.0 - handle_size * 2.0).max(0.0);
            self.handle.get_layout().0 = Offset::Static((clamped_x - handle_size).clamp(0.0, end));
            self.foreground.get_layout().2 = Size::Static(clamped_x);
        }

        let areas: Vec<_> = layout.1.iter()
            .map(|(offset, branch)| Area {offset: *offset, size: branch.0}).collect();
        let mut events: Vec<Option<Box<dyn DynEvent>>> = areas.iter().map(|_| None).collect();
        let mut slots: Vec<_> = areas.iter().zip(&mut events)
            .map(|(area, event)| ChildEvent {area, event}).collect();
        incoming.pass_dyn(ctx, &mut slots);
        let children: [&mut dyn Drawable; 3] = [
            &mut self.background, &mut self.foreground, &mut self.handle,
        ];
        for ((event, child), (_, branch)) in events.into_iter().zip(children).zip(&layout.1) {
            if let Some(event) = event {child.event(ctx, branch, event);}
        }
    }
}

impl std::fmt::Debug for _Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {f.write_str("_Slider")}
}
