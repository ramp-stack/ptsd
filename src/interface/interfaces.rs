use dyn_clone::clone_trait_object;
use prism::Context;
use prism::drawable::{Component, Drawable, Instruction, LayoutTree, RangeTree, Rect};
use prism::events::{ChildEvent, DynEvent, Event, OnEvent, TickEvent, on_event};

use crate::display::Opt;
use crate::interface::navigation::{NavigationEvent, Pages};
use crate::layout::{Column, Offset, Padding, Row, Size};

#[derive(Debug, Clone)]
pub struct ShowKeyboard(pub bool);

impl Event for ShowKeyboard {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
    }
}

pub trait Body: Drawable {
    fn pages(&mut self) -> &mut Pages;
}
clone_trait_object!(Body);

impl Drawable for Box<dyn Body> {
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

pub trait Navigator: Drawable {}
clone_trait_object!(Navigator);

impl Drawable for Box<dyn Navigator> {
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

#[derive(Component, Clone, Debug)]
pub enum Interface {
    Mobile {
        layout: Column,
        body: Box<dyn Body>,
        navigator: Option<Opt<Box<dyn Navigator>>>,
        keyboard: Opt<Box<dyn Drawable>>,
    },
    Desktop {
        layout: Row,
        navigator: Option<Opt<Box<dyn Navigator>>>,
        body: Box<dyn Body>,
    },
    Web {
        layout: Column,
        navigator: Option<Opt<Box<dyn Navigator>>>,
        body: Box<dyn Body>,
    },
}

impl Interface {
    pub fn desktop(navigator: Option<Box<dyn Navigator>>, body: impl Body) -> Self {
        Interface::Desktop {
            layout: Row::new(0.0, Offset::Start, Size::Fit, Padding::default()),
            navigator: navigator.map(|n| Opt::new(n, true)), body: Box::new(body),
        }
    }

    pub fn mobile(navigator: Option<Box<dyn Navigator>>, body: impl Body, keyboard: impl Drawable) -> Self {
        Interface::Mobile {
            layout: Column::new(0.0, Offset::Center, Size::Fit, Padding::default(), None),
            body: Box::new(body), keyboard: Opt::new(Box::new(keyboard), false),
            navigator: navigator.map(|n| Opt::new(n, true)),
        }
    }

    pub fn web(navigator: Option<Box<dyn Navigator>>, body: impl Body) -> Self {
        Interface::Web {
            layout: Column::new(0.0, Offset::Start, Size::Fill, Padding::default(), None),
            navigator: navigator.map(|n| Opt::new(n, true)), body: Box::new(body),
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

#[on_event]
impl OnEvent<NavigationEvent> for Interface {
    fn on(&mut self, _ctx: &mut Context, mut event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {
        if let Interface::Mobile {keyboard, ..} = self {keyboard.display(false);}
        if let NavigationEvent::Push(_, excluded) = &mut event {
            *excluded = match self {
                Interface::Desktop {navigator: Some(_), ..}
                | Interface::Web {navigator: Some(_), ..} => vec![0],
                _ => vec![],
            };
        }
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<ShowKeyboard> for Interface {
    fn on(&mut self, _ctx: &mut Context, event: ShowKeyboard) -> Vec<Box<dyn DynEvent>> {
        if let Interface::Mobile {keyboard, ..} = self {keyboard.display(event.0);}
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<TickEvent> for Interface {
    fn on(&mut self, _ctx: &mut Context, event: TickEvent) -> Vec<Box<dyn DynEvent>> {
        if cfg!(any(target_os = "ios", target_os = "android"))
        && let Interface::Mobile {keyboard, body, navigator: Some(navigator), ..} = self {
            navigator.display(!keyboard.is_showing() && body.pages().is_root());
        }
        prism::events![event]
    }
}
