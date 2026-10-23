use dyn_clone::clone_trait_object;
use prism::Context;
use prism::drawable::{Component, Drawable, Instruction, LayoutTree, RangeTree, Rect};
use prism::events::{ChildEvent, DynEvent, Event, EventHandler, OnEvent, WindowEvent, on_event};
use prism::hardware::Keyboard;

use std::any::Any;

use crate::display::Opt;
use crate::interface::navigation::{NavigationEvent, NavigationChanged, Pages};
use crate::layout::{Column, Offset, Padding, Row, Size};

#[derive(Debug, Clone)]
pub struct ShowKeyboard(pub bool);

impl Event for ShowKeyboard {
    fn pass(&self, _ctx: &mut Context, children: &mut [ChildEvent<'_, Self>]) {
        for child in children {*child.event = Some(self.clone());}
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
    fn listen(&mut self) -> prism::events::ListenFuture<'_> {(**self).listen()}
    fn update(&mut self, ctx: &mut Context, path: Vec<usize>, update: Box<dyn Any>) {
        (**self).update(ctx, path, update);
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

pub trait Navigator: Drawable {}
clone_trait_object!(Navigator);

impl Drawable for Box<dyn Navigator> {
    fn listen(&mut self) -> prism::events::ListenFuture<'_> {(**self).listen()}
    fn update(&mut self, ctx: &mut Context, path: Vec<usize>, update: Box<dyn Any>) {
        (**self).update(ctx, path, update);
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

#[derive(Component, Clone, Debug)]
#[on_dyn_event]
pub enum Interface {
    Mobile {
        layout: Column,
        body: Box<dyn Body>,
        navigator: Option<Opt<Box<dyn Navigator>>>,
        keyboard: Option<Opt<Box<dyn Drawable>>>,
        #[skip] native_keyboard: Option<(Keyboard, bool)>,
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

    /// Uses the supplied custom keyboard, or opens the native keyboard on `ShowKeyboard(true)`.
    pub fn mobile(navigator: Option<Box<dyn Navigator>>, mut body: impl Body, keyboard: Option<Box<dyn Drawable>>) -> Self {
        let root = body.pages().is_root();
        Interface::Mobile {
            layout: Column::center(0.0),
            body: Box::new(body), keyboard: keyboard.map(|keyboard| Opt::new(keyboard, false)),
            navigator: navigator.map(|n| Opt::new(n, root)),
            native_keyboard: None,
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

    fn show_keyboard(&mut self, ctx: &mut Context, showing: bool) {
        if let Interface::Mobile {keyboard, native_keyboard, ..} = self {
            if let Some(keyboard) = keyboard {
                if keyboard.is_showing() != showing {
                    keyboard.display(showing);
                    ctx.emit(CustomKeyboardVisibilityChanged(showing));
                }
            } else if showing {
                if native_keyboard.is_none() {
                    *native_keyboard = Some((ctx.hardware.open_keyboard(), ctx.window.keyboard_visible));
                }
            } else {
                *native_keyboard = None;
            }
        }
        self.refresh_navigator(ctx);
    }

    fn refresh_navigator(&mut self, ctx: &Context) {
        if let Interface::Mobile {
            keyboard, native_keyboard, body, navigator: Some(navigator), ..
        } = self {
            let showing = keyboard.as_ref().is_some_and(Opt::is_showing)
                || native_keyboard.is_some() || ctx.window.keyboard_visible;
            navigator.display(!showing && body.pages().is_root());
        }
    }

    fn on_event(&mut self, ctx: &mut Context, event: Box<dyn DynEvent>) -> Vec<Box<dyn DynEvent>> {
        self.refresh_navigator(ctx);
        EventHandler::dispatch(self, ctx, event)
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
    fn on(&mut self, ctx: &mut Context, mut event: NavigationEvent) -> Vec<Box<dyn DynEvent>> {
        self.show_keyboard(ctx, false);
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
        self.show_keyboard(ctx, event.0);
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

#[on_event]
impl OnEvent<WindowEvent> for Interface {
    fn on(&mut self, ctx: &mut Context, event: WindowEvent) -> Vec<Box<dyn DynEvent>> {
        if let WindowEvent::Resized(window) = &event
            && let Interface::Mobile {native_keyboard, ..} = self
            && let Some((_, visible)) = native_keyboard {
            // Resizes before the keyboard appears must not cancel the pending request.
            if *visible && !window.keyboard_visible {*native_keyboard = None;}
            else {*visible = window.keyboard_visible;}
        }
        self.refresh_navigator(ctx);
        prism::events![event]
    }
}
