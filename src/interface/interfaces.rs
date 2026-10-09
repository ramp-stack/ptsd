use prism::drawable::{Component, Drawable, SizedTree, RequestTree, Rect, DynClone, clone_trait_object};
use prism::{Context, IS_MOBILE};
use prism::event::{OnEvent, Event, TickEvent};
use prism::layout::{Area, Column, Offset, Padding, Size, Row, Stack, SizeRequest};
use prism::display::{Opt, Bin};
use prism::canvas::{Area as CanvasArea, Item as CanvasItem, Instruction, ShapeType, Shape};

use crate::Color;
use crate::interface::navigation::Pages;
use crate::navigation::NavigationEvent;

#[derive(Component, Clone, Debug)]
pub enum Interface {
    Mobile {
        layout: Column,
        safe_area_top: Bin<Stack, Rectangle>,
        body: Box<dyn Body>,
        keyboard: Opt<Box<dyn Drawable>>,
        navigator: Option<Opt<Box<dyn Navigator>>>,
        safe_area_bottom: Bin<Stack, Rectangle>,
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
    fn on_event(&mut self, _ctx: &mut Context, _sized: &SizedTree, mut event: Box<dyn Event>) -> Vec<Box<dyn Event>> {
        if event.downcast_mut::<NavigationEvent>().is_some() && let Interface::Mobile{keyboard, ..} = self {
            keyboard.display(false);
        }
        
        if let Some(NavigationEvent::Push(_, v)) = event.downcast_mut::<NavigationEvent>() {
            *v = if let Interface::Desktop{navigator,..} = self && navigator.is_some() {
                vec![0]
            } else {
                vec![]
            };

            *v = if let Interface::Mobile{..} = self {
                vec![0]
            } else {
                vec![]
            };
        }

        if let Interface::Mobile{keyboard, ..} = self 
        && let Some(ShowKeyboard(b)) = event.downcast_ref::<ShowKeyboard>() {
            keyboard.display(*b);
        }

        if IS_MOBILE && let Interface::Mobile{keyboard, body, navigator, ..} = self && event.downcast_ref::<TickEvent>().is_some() {
            let is_root = body.pages().is_root();
            if let Some(s) = navigator.as_mut() { 
                if keyboard.is_showing() {
                    s.display(false);
                } else {
                    s.display(is_root);
                }
            }
        }

        vec![event]
    }
}

impl Interface {
    pub fn desktop(ctx: &mut Context, navigator: Option<Box<dyn Navigator>>, body: impl Body + 'static) -> Self {
        let (b, l, t, r) = ctx.get_safe_area();
        Interface::Desktop {
            layout: Row::new(0.0, Offset::Start, Size::Fit, Padding(l, t, r, b)), 
            navigator: navigator.map(|n| Opt::new(n, true)),
            body: Box::new(body)
        }
    }

    pub fn mobile(ctx: &mut Context, navigator: Option<Box<dyn Navigator>>, body: impl Body + 'static, keyboard: impl Drawable + 'static) -> Self {
        let (b, l, t, r) = ctx.get_safe_area();
        let safe_area_top = Rectangle::new(Color::BLACK);
        let safe_area_top_layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(t), Padding::default());
        let safe_area_bottom = Rectangle::new(Color::BLACK);
        let safe_area_bottom_layout = Stack(Offset::Center, Offset::Center, Size::Fill, Size::Static(b), Padding::default());
        Interface::Mobile {
            layout: Column::new(0.0, Offset::Center, Size::Fit, Padding::default(), None),
            safe_area_top: Bin(safe_area_top_layout, safe_area_top),
            body: Box::new(body),
            keyboard: Opt::new(Box::new(keyboard), false),
            navigator: navigator.map(|n| Opt::new(n, true)),
            safe_area_bottom: Bin(safe_area_bottom_layout, safe_area_bottom),
        }
    }

    pub fn web(ctx: &mut Context, navigator: Option<Box<dyn Navigator>>, body: impl Body + 'static) -> Self {
        let (b, l, t, r) = ctx.get_safe_area();
        let layout = Column::new(0.0, Offset::Start, Size::Fill, Padding(l, t, r, b), None);
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
    fn build(&self, size: (f32, f32), request: &RequestTree) -> SizedTree {
        Drawable::build(&**self, size, request)
    }
    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        Drawable::draw(&**self, sized, offset, bound)
    }

    fn name(&self) -> String {Drawable::name(&**self)}

    fn event(&mut self, ctx: &mut Context, sized: &SizedTree, event: Box<dyn Event>) {
        Drawable::event(&mut **self, ctx, sized, event)
    }
}

#[derive(Debug, Clone)]
struct Rectangle(Shape);

impl Rectangle {
    fn new(color: Color) -> Self {
        Rectangle(Shape{shape: ShapeType::Rectangle(0.0, (0.0, 0.0), 0.0), color: color.into()})
    }

    fn shape(&mut self) -> &mut Shape { &mut self.0 }
}

impl Drawable for Rectangle {
    fn request_size(&self) -> RequestTree {RequestTree(SizeRequest::fill(), vec![])}

    fn draw(&self, sized: &SizedTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        let shape = match self.0.shape {
            ShapeType::RoundedRectangle(s, _, a, r) => ShapeType::RoundedRectangle(s, sized.0, a, r),
            ShapeType::Rectangle(s, _, a) => ShapeType::Rectangle(s, sized.0, a),
            ShapeType::Ellipse(s, _, a) => ShapeType::Ellipse(s, sized.0, a),
        };

        vec![Instruction(CanvasArea{offset, bounds: Some(bound)}, CanvasItem::Shape(Shape{shape, color: self.0.color}))]
    }
}