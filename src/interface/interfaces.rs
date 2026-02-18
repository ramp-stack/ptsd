use prism::drawable::{Component, Drawable, SizedTree, RequestTree, Rect, DynClone, clone_trait_object};
use prism::Context;
use prism::event::{OnEvent, Event};
use prism::layout::{Area, Column, Offset, Padding, Size, Row};
use prism::display::Opt;
use prism::canvas::{Area as CanvasArea, Item as CanvasItem};

use crate::interface::navigation::Pages;

#[derive(Component, Clone, Debug)]
pub enum Interface {
    Mobile {
        layout: Column,
        body: Box<dyn Body>,
        keyboard: Opt<Box<dyn Drawable>>,
        navigator: Option<Opt<Box<dyn Navigator>>>,
    },

    Desktop {
        layout: Row, 
        navigator: Option<Opt<Box<dyn Navigator>>>,
        body: Box<dyn Body> 
    },

    Web {
        layout: Column, 
        navigator: Option<Opt<Box<dyn Navigator>>>, 
        body: Box<dyn Body>
    }
}


impl OnEvent for Interface {
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        // if let Some(NavigationEvent::Push(_, v)) = event.downcast_mut::<NavigationEvent>() {
        //     println!("EVENT");
        //     if let Interface::Mobile{..} = self {*v = vec![2, 3];}  
        //     *v = vec![1];
        // }

        if let Interface::Mobile{keyboard, ..} = self 
        && let Some(ShowKeyboard(b)) = event.downcast_ref::<ShowKeyboard>() {
            keyboard.display(*b);
        }

        vec![event]
    }
}

impl Interface {
    pub fn desktop(navigator: Option<Box<dyn Navigator>>, body: impl Body + 'static) -> Self {
        Interface::Desktop {
            layout: Row::start(0.0), 
            navigator: navigator.map(|n| Opt::new(n, true)),
            body: Box::new(body)
        }
    }

    pub fn mobile(navigator: Option<Box<dyn Navigator>>, body: impl Body + 'static, keyboard: impl Drawable + 'static) -> Self {
        // let (_left, _right, top, bottom) = ctx.send(Request::Hardware(Hardware::SafeAreaInsets));
        let (top, bottom) = (18.0, 18.0);
        let layout = Column::new(0.0, Offset::Center, Size::Fit, Padding(0.0, top, 0.0, bottom), None);

        Interface::Mobile {
            layout,
            body: Box::new(body),
            keyboard: Opt::new(Box::new(keyboard), false),
            navigator: navigator.map(|n| Opt::new(n, true)),
        }
    }

    pub fn web(navigator: Option<Box<dyn Navigator>>, body: impl Body + 'static) -> Self {
        let layout = Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None);
        Interface::Web {
            layout, 
            navigator: navigator.map(|n| Opt::new(n, true)),
            body: Box::new(body),
        }
    }

    pub fn pages(&mut self) -> &mut Pages {
        match self {
            Interface::Desktop {body, ..} => body.pages(),
            Interface::Mobile {body, ..} => body.pages(),
            Interface::Web {body, ..} => body.pages(),
        }
    }

    pub fn navigator(&mut self) -> &mut Option<Opt<Box<dyn Navigator>>> {
        match self {
            Interface::Desktop {navigator, ..} => navigator,
            Interface::Mobile {navigator, ..} => navigator,
            Interface::Web {navigator, ..} => navigator,
        }
    }
}

/// Event used to open or close keyboard.
#[derive(Debug, Clone)]
pub struct ShowKeyboard(pub bool);

impl Event for ShowKeyboard {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: &[Area]) -> Vec<Option<Box<dyn Event>>> {
        children.iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

pub trait Body: Drawable + DynClone + std::fmt::Debug + 'static {
    fn pages(&mut self) -> &mut Pages;
}

clone_trait_object!(Body);

pub trait Navigator: Drawable + DynClone + std::fmt::Debug + 'static {}

clone_trait_object!(Navigator);

impl Drawable for Box<dyn Navigator> {
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


