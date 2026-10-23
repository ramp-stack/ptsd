use dyn_clone::clone_trait_object;
use prism::Context;
use prism::drawable::{Caret, Component, Drawable, Instruction, LayoutTree, RangeTree, Rect};
use prism::events::{ChangeFuture, ChildEvent, DynEvent, Event, OnEvent, on_event};

use crate::display::{EitherOr, Enum};
use crate::layout::{Offset, Padding, Size, Stack};
use crate::utils::Callback;

#[derive(Debug, Clone)]
pub enum NavigationEvent {
    Pop,
    Push(Option<Box<dyn FlowContainer>>, Vec<usize>),
    Restart(Option<Box<dyn FlowContainer>>),
    ResetWithFunction(Box<dyn Callback>),
    Reset,
    Root(Option<String>),
    Error(String),
    Next,
}

impl NavigationEvent {
    pub fn root(root: &str) -> Self {NavigationEvent::Root(Some(root.to_string()))}
    pub fn push(flow: impl FlowContainer) -> Self {
        NavigationEvent::Push(Some(Box::new(flow)), vec![])
    }
    pub fn restart(flow: impl FlowContainer) -> Self {NavigationEvent::Restart(Some(Box::new(flow)))}
    pub fn reset_with_fn(callback: impl Callback) -> Self {
        NavigationEvent::ResetWithFunction(Box::new(callback))
    }
}

impl Event for NavigationEvent {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        if let NavigationEvent::Push(Some(flow), excluded) = self {
            let mut passed = false;
            for (index, child) in children.iter_mut().enumerate() {
                *child.event = if !passed && !excluded.contains(&index) {
                    passed = true;
                    Some(NavigationEvent::Push(Some(flow.clone()), vec![]))
                } else {None};
            }
        } else {
            for child in children {*child.event = Some(self.clone());}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NavigationChanged;
impl Event for NavigationChanged {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

pub trait FlowContainer: Drawable {
    fn flow(&mut self) -> &mut Flow;
}
clone_trait_object!(FlowContainer);

impl Drawable for Box<dyn FlowContainer> {
    fn caret(&self, layout: &LayoutTree) -> Option<Caret> {Drawable::caret(&**self, layout)}
    fn changed(&mut self) -> ChangeFuture<'_> {
        Box::pin(async move {(**self).changed().await.map::<Self>(|boxed| &mut **boxed)})
    }
    fn size_range(&self) -> RangeTree {Drawable::size_range(&**self)}
    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        Drawable::layout(&mut **self, size, range)
    }
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        Drawable::draw(&**self, layout, offset, bound)
    }
    fn name(&self) -> String {Drawable::name(&**self)}
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        Drawable::event(&mut **self, ctx, layout, event)
    }
}

pub trait AppPage: Drawable {}
downcast_rs::impl_downcast!(AppPage);
clone_trait_object!(AppPage);

impl Drawable for Box<dyn AppPage> {
    fn caret(&self, layout: &LayoutTree) -> Option<Caret> {Drawable::caret(&**self, layout)}
    fn changed(&mut self) -> ChangeFuture<'_> {
        Box::pin(async move {(**self).changed().await.map::<Self>(|boxed| &mut **boxed)})
    }
    fn size_range(&self) -> RangeTree {Drawable::size_range(&**self)}
    fn layout(&mut self, size: (f32, f32), range: &RangeTree) -> LayoutTree {
        Drawable::layout(&mut **self, size, range)
    }
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        Drawable::draw(&**self, layout, offset, bound)
    }
    fn name(&self) -> String {Drawable::name(&**self)}
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        Drawable::event(&mut **self, ctx, layout, event)
    }
}

#[derive(Debug, Component, Clone)]
pub struct Flow {
    layout: Stack,
    pub current: Option<Box<dyn AppPage>>,
    #[skip] pub stored: Vec<Box<dyn AppPage>>,
    #[skip] pub index: usize,
}

impl Flow {
    pub fn new(mut pages: Vec<Box<dyn AppPage>>) -> Self {
        Flow {layout: Stack::default(), current: Some(pages.remove(0)), stored: pages, index: 0}
    }
}

#[on_event]
impl OnEvent<NavigationEvent> for Flow {
    fn on(&mut self, ctx: &mut Context, event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {
        let previous = self.index;
        match &event {
            NavigationEvent::Pop => {
                if self.index == 0 {ctx.emit(NavigationEvent::Reset);} else {self.index -= 1;}
            },
            NavigationEvent::Next if self.index < self.stored.len() => self.index += 1,
            _ => {},
        }
        self.stored.insert(previous, self.current.take().unwrap());
        if self.stored.get(self.index).is_some() {
            self.current = Some(self.stored.remove(self.index));
        }
        if self.index != previous {ctx.emit(NavigationChanged);}
        prism::events![event]
    }
}

#[derive(Debug, Clone, Component)]
pub struct History(Stack, Vec<Box<dyn FlowContainer>>);

impl History {
    pub fn new(history: Vec<Box<dyn FlowContainer>>) -> Self {
        History(Stack::new(
            Offset::Start, Offset::Start, Size::Static(0.0), Size::Static(0.0), Padding::default()
        ), history)
    }
    pub fn inner(&mut self) -> &mut Vec<Box<dyn FlowContainer>> {&mut self.1}
}

#[on_event]
impl OnEvent<NavigationEvent> for History {
    fn on(&mut self, _ctx: &mut Context, _event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {vec![]}
}

#[derive(Debug, Component, Clone)]
pub struct Pages {
    layout: Stack,
    inner: EitherOr<Enum<Box<dyn AppPage>>, Option<Box<dyn FlowContainer>>>,
    #[skip] history: History,
}

impl Pages {
    pub fn new(roots: Vec<(String, Box<dyn AppPage>)>) -> Self {
        let first = roots[0].0.to_string();
        let roots = Enum::new(roots, first);
        Pages {
            layout: Stack::default(), inner: EitherOr::new(roots, None),
            history: History::new(Vec::new()),
        }
    }

    pub fn root(&mut self, ctx: &mut Context, page: Option<String>) {
        self.inner.display_left(true);
        if let Some(page) = page {self.inner.left().display(&page);}
        *self.history.inner() = vec![];
        *self.inner.right() = None;
        ctx.emit(NavigationChanged);
    }

    pub fn try_back(&mut self, ctx: &mut Context) {
        if let Some(flow) = self.history.inner().pop() {
            self.inner.right().replace(flow);
            ctx.emit(NavigationChanged);
        } else {
            self.root(ctx, None);
        }
    }

    pub fn push(&mut self, ctx: &mut Context, flow: Box<dyn FlowContainer>) {
        if let Some(old) = self.inner.right().replace(flow) {self.history.inner().push(old);}
        self.inner.display_left(false);
        ctx.emit(NavigationChanged);
    }
    pub fn restart(&mut self, ctx: &mut Context, flow: Box<dyn FlowContainer>) {
        *self.inner.right() = Some(flow);
        self.inner.display_left(false);
        ctx.emit(NavigationChanged);
    }

    pub fn current(&mut self) -> &mut Box<dyn AppPage> {
        if !self.history.inner().is_empty() || self.inner.right().is_some() {
            self.inner.right().as_mut().unwrap().flow().current.as_mut().unwrap()
        } else {
            self.inner.left().drawable().inner()
        }
    }
    pub fn is_root(&self) -> bool {self.inner.is_left()}
}

#[on_event]
impl OnEvent<NavigationEvent> for Pages {
    fn on(&mut self, ctx: &mut Context, event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {
        match event {
            NavigationEvent::Push(flow, ..) => self.push(ctx, flow.unwrap()),
            NavigationEvent::Reset => self.try_back(ctx),
            NavigationEvent::ResetWithFunction(mut function) => {
                self.try_back(ctx);
                function(ctx);
            },
            NavigationEvent::Root(root) => self.root(ctx, root),
            NavigationEvent::Restart(flow) => self.restart(ctx, flow.unwrap()),
            _ => return prism::events![event],
        }
        vec![]
    }
}
