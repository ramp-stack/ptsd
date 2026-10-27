use prism::drawable::{Component, Drawables, Layout};

use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use crate::layout::Stack;

mod dismissible;
pub use dismissible::{Dismissible, DismissEdge};

#[derive(Debug, Component, Clone)]
pub struct Bin<L: Layout + Clone + 'static, D: Drawables + Clone + Debug + 'static>(pub L, pub D);

impl<L: Layout + Clone + 'static, D: Drawables + Clone + Debug + 'static> Bin<L, D> {
    pub fn inner(&mut self) -> &mut D {&mut self.1}
    pub fn get_layout(&mut self) -> &mut L {&mut self.0}
}

impl<L: Layout + Clone + 'static, D: Drawables + Clone + Debug + 'static> Deref for Bin<L, D> {
    type Target = D;
    fn deref(&self) -> &D {&self.1}
}

impl<L: Layout + Clone + 'static, D: Drawables + Clone + Debug + 'static> DerefMut for Bin<L, D> {
    fn deref_mut(&mut self) -> &mut D {&mut self.1}
}

#[derive(Debug, Component, Clone)]
pub struct Opt<D: Drawables + Clone + Debug + 'static>(Stack, Option<D>, #[skip] Option<D>);

impl<D: Drawables + Clone + Debug + 'static> Opt<D> {
    pub fn new(item: D, display: bool) -> Self {
        if display {Self(Stack::default(), Some(item), None)}
        else {Self(Stack::default(), None, Some(item))}
    }
    pub fn display(&mut self, display: bool) {
        if display == self.is_showing() {return}
        if display {self.1 = self.2.take();} else {self.2 = self.1.take();}
    }
    pub fn inner(&mut self) -> &mut D {self.1.as_mut().or(self.2.as_mut()).unwrap()}
    pub fn inner_ref(&self) -> &D {self.1.as_ref().or(self.2.as_ref()).unwrap()}
    pub fn is_showing(&self) -> bool {self.1.is_some()}
}

#[derive(Debug, Component, Clone)]
pub struct EitherOr<
    L: Drawables + Clone + Debug + 'static, R: Drawables + Clone + Debug + 'static
>(Stack, Opt<L>, Opt<R>);

impl<L: Drawables + Clone + Debug + 'static, R: Drawables + Clone + Debug + 'static> EitherOr<L, R> {
    pub fn new(left: L, right: R) -> Self {
        EitherOr(Stack::default(), Opt::new(left, true), Opt::new(right, false))
    }

    pub fn display_left(&mut self, display_left: bool) {
        self.1.display(display_left);
        self.2.display(!display_left);
    }

    pub fn left(&mut self) -> &mut L {self.1.inner()}
    pub fn right(&mut self) -> &mut R {self.2.inner()}
    pub fn is_left(&self) -> bool {self.1.is_showing()}
}

#[derive(Debug, Component, Clone)]
pub struct Enum<D: Drawables + Clone + Debug + 'static>(Stack, HashMap<String, Opt<D>>, #[skip] String);

impl<D: Drawables + Clone + Debug + 'static> Enum<D> {
    pub fn new(items: Vec<(String, D)>, start: String) -> Self {
        let items = items.into_iter().map(|(name, item)| {
            let displayed = name == start;
            (name, Opt::new(item, displayed))
        }).collect();
        Enum(Stack::default(), items, start)
    }

    pub fn display(&mut self, name: &str) -> bool {
        if self.1.contains_key(name) {
            self.2 = name.to_string();
            for (key, value) in &mut self.1 {value.display(key == name);}
            true
        } else {false}
    }

    pub fn current(&self) -> String {self.2.clone()}
    pub fn drawable(&mut self) -> &mut Opt<D> {self.1.get_mut(&self.2).unwrap()}
}
