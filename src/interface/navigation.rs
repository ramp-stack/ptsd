use prism::drawable::{Component, Drawable, SizedTree, RequestTree, Rect, DynClone, clone_trait_object};
use prism::{Context, Request};
use prism::canvas::{Area as CanvasArea, Item as CanvasItem};
use prism::event::{OnEvent, Event};
use prism::layout::{Area, Stack, Offset, Size, Padding};
use prism::display::{EitherOr, Enum};

// should this be a trait so that "FlowStorage" and other variables stay alive?
#[derive(Debug, Component, Clone)]
pub struct Flow {
    layout: Stack,
    pub current: Option<Box<dyn AppPage>>,
    #[skip] pub stored: Vec<Box<dyn AppPage>>,
    #[skip] pub index: usize
}

impl Flow {
    pub fn new(mut pages: Vec<Box<dyn AppPage>>) -> Self {
        Flow {
            layout: Stack::default(),
            current: Some(pages.remove(0)),
            stored: pages,
            index: 0
        }
    }
}

impl OnEvent for Flow {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_mut::<NavigationEvent>() {
            let i = self.index;
            match event {
                NavigationEvent::Pop => {
                    if self.index == 0 {
                        self.index = 0;
                        ctx.emit(NavigationEvent::Reset);
                    } else {
                        self.index -= 1;
                    }
                },
                NavigationEvent::Next if self.index < self.stored.len() => self.index += 1,
                _ => {}
            }
            
            self.stored.insert(i, self.current.take().unwrap()); 
        
            if self.stored.get(self.index).is_some() {
                self.current = Some(self.stored.remove(self.index));
            }
        }
        vec![event]
    }
}

#[derive(Debug, Clone, Component)]
pub struct History(Stack, Vec<Box<dyn FlowContainer>>);
impl OnEvent for History {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_ref::<NavigationEvent>().is_some() { return vec![]; }
        vec![event]
    }
}

// impl Component for History {
//     fn children(&self) -> Vec<&dyn Drawable> {vec![]}
//     fn children_mut(&mut self) -> Vec<&mut dyn Drawable> {vec![]}
//     fn layout(&self) -> &dyn prism::layout::Layout {&self.0}
// }

impl History {
    pub fn new(h: Vec<Box<dyn FlowContainer>>) -> Self {
        History(Stack::new(Offset::Start, Offset::Start, Size::Static(0.0), Size::Static(0.0), Padding::default()), h)
    }

    pub fn inner(&mut self) -> &mut Vec<Box<dyn FlowContainer>> {&mut self.1}
}

#[derive(Debug, Component, Clone)]
pub struct Pages {
    layout: Stack,
    #[allow(clippy::type_complexity)] inner: EitherOr<Enum<Box<dyn AppPage>>, Option<Box<dyn FlowContainer>>>,
    history: History,
}

impl OnEvent for Pages {
    fn on_event(&mut self, ctx: &mut Context, _sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(e) = event.downcast_mut::<NavigationEvent>() {
            ctx.stop_camera();
            match e {
                NavigationEvent::Push(flow, ..) => self.push(flow.take().unwrap()),
                NavigationEvent::Reset => self.try_back(),
                NavigationEvent::Root(root) => self.root(Some(root.to_string())),
                NavigationEvent::Restart(flow) => self.restart(flow.take().unwrap()),
                _ => {return vec![event]}
            }
            return vec![];
        }

        vec![event]
    }
}

impl Pages {
    pub fn new(roots: Vec<(String, Box<dyn AppPage>)>) -> Self {
        let first = roots[0].0.to_string();
        let roots = Enum::new(roots, first);
        Pages {
            layout: Stack::default(),
            inner: EitherOr::new(roots, None),
            history: History::new(Vec::new()),
        }
    }

    pub fn root(&mut self, page: Option<String>) {
        self.inner.display_left(true);
        if let Some(p) = page { self.inner.left().display(&p); }
        *self.history.inner() = vec![];
        *self.inner.right() = None;
    }

    pub fn try_back(&mut self) {
        if let Some(flow) = self.history.inner().pop() {
            self.inner.right().replace(flow);
        } else {
            self.root(None);
        }
    }

    pub fn push(&mut self, flow: Box<dyn FlowContainer>) {
        if let Some(old) = self.inner.right().replace(flow) { 
            self.history.inner().push(old);
        }
        self.inner.display_left(false);
    }

    pub fn restart(&mut self, flow: Box<dyn FlowContainer>) {
        self.try_back();
        self.push(flow);
    }

    pub fn current(&mut self) -> &mut Box<dyn AppPage> {
        if !self.history.inner().is_empty() || self.inner.right().is_some() {
            self.inner.right().as_mut().unwrap().flow().current.as_mut().unwrap()
        } else {
            self.inner.left().drawable().inner()
        }
    }

    pub fn is_root(&self) -> bool { self.inner.is_left() }
}

#[derive(Debug, Clone)]
pub enum NavigationEvent {
    Pop,
    Push(Option<Box<dyn FlowContainer>>, Vec<usize>),
    Restart(Option<Box<dyn FlowContainer>>),
    Reset,
    Root(String),
    Error(String),
    Next,
}

impl NavigationEvent {
    pub fn push(flow: impl FlowContainer + 'static) -> Self {
        NavigationEvent::Push(Some(Box::new(flow)), vec![])
    }

    pub fn restart(flow: impl FlowContainer + 'static) -> Self {
        NavigationEvent::Restart(Some(Box::new(flow)))
    }
}

impl Event for NavigationEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        let v = match self.as_ref() {
            NavigationEvent::Push(Some(_), v) => Some(v.clone()),
            _ => None
        };

        if v.is_none() { return children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect(); }

        let v = v.unwrap();

        let mut x = Some(self as Box<dyn Event>);
        children.iter().enumerate().map(|(i, _)| if v.contains(&i) {None} else {x.take()}).collect()
    }
}

// impl Event for NavigationEvent {
//     fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
//         println!("Children {:?}", children.len());
//         children.iter().enumerate().map(|(i, _)| match *self {
//             NavigationEvent::Push(_, ref v) => {
//                 (!v.contains(&i)).then(|| {
//                     let mut e: NavigationEvent = (*self).clone();
//                     if let NavigationEvent::Push(_, ref mut v2) = e {v2.clear();}
//                     Box::new(e) as Box<dyn Event>
//                 })
//             },
//             _ =>  Some(Box::new((*self).clone()) as Box<dyn Event>),
//         }).collect()
//     }
// }

pub trait FlowContainer: Drawable + DynClone + std::fmt::Debug + 'static {
    fn flow(&mut self) -> &mut Flow;
}

clone_trait_object!(FlowContainer);

impl Drawable for Box<dyn FlowContainer> {
    fn request_size(&self) -> RequestTree {Drawable::request_size(&**self)}
    fn build(&self, size: (f32, f32), request: RequestTree) -> SizedTree {
        Drawable::build(&**self, size, request)
    }
    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<(CanvasArea, CanvasItem)> {
        Drawable::draw(&**self, sized, offset, bound)
    }

    fn name(&self) -> String {Drawable::name(&**self)}

    fn event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) {
        Drawable::event(&mut **self, ctx, sized, event)
    }
}

pub trait AppPage: Drawable + DynClone + std::fmt::Debug + 'static {}

downcast_rs::impl_downcast!(AppPage);
impl Drawable for Box<dyn AppPage> {
    fn request_size(&self) -> RequestTree {Drawable::request_size(&**self)}
    fn build(&self, size: (f32, f32), request: RequestTree) -> SizedTree {
        Drawable::build(&**self, size, request)
    }
    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<(CanvasArea, CanvasItem)> {
        Drawable::draw(&**self, sized, offset, bound)
    }

    fn name(&self) -> String {Drawable::name(&**self)}

    fn event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) {
        Drawable::event(&mut **self, ctx, sized, event)
    }
}

clone_trait_object!(AppPage);
