use prism::drawable::{Component, Drawable, SizedTree, RequestTree, Rect};
use prism::{Context, Request};
use prism::canvas::{Area as CanvasArea, Item as CanvasItem};
use prism::event::{OnEvent, Event};
use prism::layout::{Area, Column, Offset, Padding, Size, Stack, Row};
use prism::display::{Bin, Opt, EitherOr, Enum};

// should this be a trait so that "FlowStorage" and other variables stay alive?
#[derive(Debug, Component)]
pub struct Flow {
    layout: Stack,
    current: Option<Box<dyn Drawable>>,
    #[skip] stored: Vec<Box<dyn Drawable>>,
    #[skip] index: usize
}

impl Flow {
    pub fn new(mut pages: Vec<Box<dyn Drawable>>) -> Self {
        Flow {
            layout: Stack::default(),
            current: Some(pages.remove(0)),
            stored: pages,
            index: 0
        }
    }
}

impl OnEvent for Flow {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // println!("Event {:?}", event);
        if let Some(event) = event.downcast_mut::<NavigationEvent>() {
            println!("received {:?}", event);
            let i = self.index;
            match event {
                NavigationEvent::Pop => {
                    if self.index <= 0 {
                        self.index = 0;
                        ctx.send(Request::event(NavigationEvent::Reset));
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

// interface contains root pages and flows. 

#[derive(Debug, Component)]
pub struct Pages {
    layout: Stack,
    inner: EitherOr<Enum, Option<Box<dyn FlowContainer>>>, // enum is root pages, otherwise show a flow (if left is showing, hide the navigator)
    #[skip] history: Vec<Box<dyn FlowContainer>>
}

impl OnEvent for Pages {
    fn on_event(&mut self, ctx: &mut Context, sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if let Some(event) = event.downcast_mut::<NavigationEvent>() {
            match event {
                NavigationEvent::Push(flow) => self.push(flow.take().unwrap()),
                NavigationEvent::Reset => self.root(None),
                NavigationEvent::Root(root) => self.root(Some(root.to_string())),
                _ => {}
            }
        }

        vec![event]
    }
}

impl Pages {
    pub fn new(roots: Vec<(String, Box<dyn Drawable>)>) -> Self {
        let first = roots[0].0.to_string();
        let roots = Enum::new(roots, first);
        Pages {
            layout: Stack::default(),
            inner: EitherOr::new(roots, None),
            history: Vec::new(),
        }
    }

    pub fn root(&mut self, page: Option<String>) {
        self.inner.display_left(true);
        if let Some(p) = page { self.inner.left().display(&p); }
        self.history = vec![];
        *self.inner.right() = None;
    }

    pub fn push(&mut self, flow: Box<dyn FlowContainer>) {
        if let Some(old) = self.inner.right().replace(flow) { 
            self.history.push(old);
        }
        self.inner.display_left(false);
    }

    pub fn current(&mut self) -> &mut Box<dyn Drawable> {
        if !self.history.is_empty() || self.inner.right().is_some() {
            self.inner.right().as_mut().unwrap().flow().current.as_mut().unwrap()
        } else {
            self.inner.left().drawable().inner()
        }
    }
}

pub enum NavigationEvent {
    Pop,
    Push(Option<Box<dyn FlowContainer>>),
    Reset,
    Root(String),
    Error(String),
    Next,
}

impl std::fmt::Debug for NavigationEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", match self {
            NavigationEvent::Pop => "Pop".to_string(),
            NavigationEvent::Push(..) => "Push".to_string(),
            NavigationEvent::Reset => "Reset".to_string(),
            NavigationEvent::Root(r) => format!("Root({r})"),
            NavigationEvent::Error(e) => format!("Error({e})"),
            NavigationEvent::Next => "Next".to_string(),
        })
    }
}

impl Clone for NavigationEvent {
    fn clone(&self) -> Self {
        match self {
            NavigationEvent::Pop => NavigationEvent::Pop,
            NavigationEvent::Push(_) => NavigationEvent::Push(None), // drop payload on clone
            NavigationEvent::Reset => NavigationEvent::Reset,
            NavigationEvent::Root(s) => NavigationEvent::Root(s.clone()),
            NavigationEvent::Error(s) => NavigationEvent::Error(s.clone()),
            NavigationEvent::Next => NavigationEvent::Next,
        }
    }
}

impl NavigationEvent {
    pub fn push(flow: impl FlowContainer + 'static) -> Self {
        NavigationEvent::Push(Some(Box::new(flow)))
    }
}

impl Event for NavigationEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        // vec![Some(Box::new(_NavEvent(self)))]
        
        let mut events = vec![];
        children.iter().for_each(|_| events.push(Some(self.clone() as Box<dyn Event>)));
        events.push(Some(self as Box<dyn Event>));
        events.into_iter().rev().collect::<Vec<_>>()
    }
}


pub trait FlowContainer: Drawable + std::fmt::Debug + 'static {
    fn flow(&mut self) -> &mut Flow;
}

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

pub trait AppPage: Drawable + std::fmt::Debug + 'static {}

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

