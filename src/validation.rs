use dyn_clone::DynClone;
use prism::Context;

use std::ops::Deref;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

/// The dependency supplied to a watched validator. Source methods are available through `Deref`.
pub struct Watched<S, U> {
    source: S,
    latest: Option<Rc<U>>,
}
impl<S, U> Watched<S, U> {
    /// The last delivered update, retained across input edits and subsequent waits.
    pub fn latest(&self) -> Option<&U> {self.latest.as_deref()}
}
impl<S: Clone, U> Clone for Watched<S, U> {
    fn clone(&self) -> Self {Self {source: self.source.clone(), latest: self.latest.clone()}}
}
impl<S, U> Deref for Watched<S, U> {
    type Target = S;
    fn deref(&self) -> &S {&self.source}
}

/// Validation and optional dependency notifications share one owner.
pub trait Validate<I, O>: DynClone + 'static {
    fn validate(&mut self, ctx: &mut Context, input: I) -> O;
    fn change(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + '_>> {Box::pin(async {None})}
}
impl<I, O, F: FnMut(&mut Context, I) -> O + Clone + 'static> Validate<I, O> for F {
    fn validate(&mut self, ctx: &mut Context, input: I) -> O {self(ctx, input)}
}
impl<
    I, O, S: Clone + 'static, U: 'static, L: AsyncFnMut(&mut S) -> U + Clone + 'static,
    F: FnMut(&mut Context, &Watched<S, U>, I) -> O + Clone + 'static,
> Validate<I, O> for Validator<S, U, L, F> {
    fn validate(&mut self, ctx: &mut Context, input: I) -> O {(self.validate)(ctx, &self.source, input)}
    fn change(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + '_>> {
        Box::pin(async {
            self.source.latest = Some(Rc::new((self.listen)(&mut self.source.source).await));
            Some(())
        })
    }
}

/// A validation callback with an explicitly watched dependency.
pub struct Validator<S, U, L, F> {
    source: Watched<S, U>,
    listen: L,
    validate: F,
}
impl<S, U, L: AsyncFnMut(&mut S) -> U, F> Validator<S, U, L, F> {
    /// `listen` waits for an update; `validate` receives the source and its latest result.
    pub fn watch(source: S, listen: L, validate: F) -> Self {
        Self {source: Watched {source, latest: None}, listen, validate}
    }
}
impl<S: Clone, U, L: Clone, F: Clone> Clone for Validator<S, U, L, F> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(), listen: self.listen.clone(), validate: self.validate.clone(),
        }
    }
}
