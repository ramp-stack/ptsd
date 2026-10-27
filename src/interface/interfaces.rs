use dyn_clone::clone_trait_object;
use prism::Context;
use prism::drawable::{Component, Drawable, Instruction, LayoutTree, Rect};
use prism::events::{
    ChildEvent, DynEvent, Event, ListenCallback, ListenFuture, OnEvent, WindowEvent, on_event,
};

use std::any::Any;

use crate::display::Opt;
use crate::interface::navigation::{NavigationEvent, NavigationChanged, Pages};
use crate::layout::{Column, Offset, Padding, Row, Size};

#[derive(Debug, Clone, Copy)]
pub enum ShowKeyboard {Show {dismiss_padding: f32}, Hide}

impl Event for ShowKeyboard {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CustomKeyboardVisibilityChanged(pub bool);
impl Event for CustomKeyboardVisibilityChanged {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(*self);}
    }
}

pub trait Body: Drawable {
    fn pages(&mut self) -> &mut Pages;
}
clone_trait_object!(Body);

impl Drawable for Box<dyn Body> {
    fn name(&self) -> String {Drawable::name(&**self)}
    fn request_width(&mut self) -> (f32, f32) {Drawable::request_width(&mut **self)}
    fn request_height(&mut self, width: f32) -> (f32, f32) {
        Drawable::request_height(&mut **self, width)
    }
    fn layout(&mut self, size: (f32, f32)) -> LayoutTree {Drawable::layout(&mut **self, size)}
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        Drawable::draw(&**self, layout, offset, bound)
    }
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        Drawable::event(&mut **self, ctx, layout, event)
    }
    fn changed(&mut self) -> ListenFuture<'_> {
        Box::pin(async move {
            let callback = (**self).changed().await;
            Box::new(move |this: &mut dyn Any, ctx: &mut Context| {
                callback(this.downcast_mut::<Self>().unwrap().as_mut().as_any_mut(), ctx);
            }) as ListenCallback
        })
    }
}

pub trait Navigator: Drawable {}
clone_trait_object!(Navigator);

impl Drawable for Box<dyn Navigator> {
    fn name(&self) -> String {Drawable::name(&**self)}
    fn request_width(&mut self) -> (f32, f32) {Drawable::request_width(&mut **self)}
    fn request_height(&mut self, width: f32) -> (f32, f32) {
        Drawable::request_height(&mut **self, width)
    }
    fn layout(&mut self, size: (f32, f32)) -> LayoutTree {Drawable::layout(&mut **self, size)}
    fn draw(&self, layout: &LayoutTree, offset: (f32, f32), bound: Rect) -> Vec<Instruction> {
        Drawable::draw(&**self, layout, offset, bound)
    }
    fn event(&mut self, ctx: &mut Context, layout: &LayoutTree, event: Box<dyn DynEvent>) {
        Drawable::event(&mut **self, ctx, layout, event)
    }
    fn changed(&mut self) -> ListenFuture<'_> {
        Box::pin(async move {
            let callback = (**self).changed().await;
            Box::new(move |this: &mut dyn Any, ctx: &mut Context| {
                callback(this.downcast_mut::<Self>().unwrap().as_mut().as_any_mut(), ctx);
            }) as ListenCallback
        })
    }
}

#[derive(Component, Clone, Debug)]
pub enum Interface {
    Mobile {
        layout: Column,
        body: Box<dyn Body>,
        navigator: Option<Opt<Box<dyn Navigator>>>,
        keyboard: Option<Opt<Box<dyn Drawable>>>,
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

    /// Uses the supplied custom keyboard, or shows and hides the shared native keyboard handle.
    /// Hardware owns the handle; visibility changes preserve the focused input.
    pub fn mobile(navigator: Option<Box<dyn Navigator>>, mut body: impl Body, keyboard: Option<Box<dyn Drawable>>) -> Self {
        let root = body.pages().is_root();
        Interface::Mobile {
            layout: Column::center(0.0),
            body: Box::new(body), keyboard: keyboard.map(|keyboard| Opt::new(keyboard, false)),
            navigator: navigator.map(|n| Opt::new(n, root)),
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

    fn show_keyboard(&mut self, ctx: &mut Context, event: ShowKeyboard) {
        let showing = matches!(event, ShowKeyboard::Show {..});
        if let Interface::Mobile {keyboard, ..} = self {
            if let Some(keyboard) = keyboard {
                if keyboard.is_showing() != showing {
                    keyboard.display(showing);
                    ctx.emit(CustomKeyboardVisibilityChanged(showing));
                }
            } else if let ShowKeyboard::Show {dismiss_padding} = event {
                ctx.hardware.keyboard().show(dismiss_padding);
            } else {ctx.hardware.keyboard().hide();}
        }
        self.refresh_navigator(ctx);
    }

    fn refresh_navigator(&mut self, ctx: &Context) {
        if let Interface::Mobile {keyboard, body, navigator: Some(navigator), ..} = self {
            let showing = keyboard.as_ref().is_some_and(Opt::is_showing)
                || ctx.hardware.keyboard().visible();
            navigator.display(!showing && body.pages().is_root());
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
impl OnEvent<WindowEvent> for Interface {
    fn on(&mut self, ctx: &mut Context, event: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if matches!(event, WindowEvent::Resized(_)) {self.refresh_navigator(ctx);}
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<NavigationEvent> for Interface {
    fn on(&mut self, ctx: &mut Context, mut event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {
        self.show_keyboard(ctx, ShowKeyboard::Hide);
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
    fn on(&mut self, ctx: &mut Context, event: ShowKeyboard) -> Vec<Box<dyn DynEvent>> {
        self.show_keyboard(ctx, event);
        prism::events![event]
    }
}

#[on_event]
impl OnEvent<NavigationChanged> for Interface {
    fn on(&mut self, ctx: &mut Context, event: NavigationChanged) -> Vec<Box<dyn DynEvent>> {
        self.refresh_navigator(ctx);
        prism::events![event]
    }
}
