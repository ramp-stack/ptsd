use dyn_clone::DynClone;
use prism::Context;
use prism::events::ChangeSource;

use std::ops::Deref;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

/// The dependency supplied to a watched validator. Source methods are available through `Deref`.
pub struct Watched<S: ChangeSource> {
    source: S,
    latest: Option<Rc<S::Update>>,
}
impl<S: ChangeSource> Watched<S> {
    /// The last delivered update, retained across input edits and source completion.
    pub fn latest(&self) -> Option<&S::Update> {self.latest.as_deref()}
}
impl<S: ChangeSource + Clone> Clone for Watched<S> {
    fn clone(&self) -> Self {Self {source: self.source.clone(), latest: self.latest.clone()}}
}
impl<S: ChangeSource> Deref for Watched<S> {
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
impl<I, O, S: ChangeSource + Clone, F: FnMut(&mut Context, &Watched<S>, I) -> O + Clone + 'static>
    Validate<I, O> for Validator<S, F> {
    fn validate(&mut self, ctx: &mut Context, input: I) -> O {(self.validate)(ctx, &self.source, input)}
    fn change(&mut self) -> Pin<Box<dyn Future<Output = Option<()>> + '_>> {Box::pin(self.next())}
}

/// A validation callback with an explicitly watched dependency.
#[derive(Clone)]
pub struct Validator<S: ChangeSource, F> {
    source: Watched<S>,
    validate: F,
}
impl<S: ChangeSource, F> Validator<S, F> {
    pub fn watch(source: S, validate: F) -> Self {
        Self {source: Watched {source, latest: None}, validate}
    }
}
impl<S: ChangeSource, F: 'static> ChangeSource for Validator<S, F> {
    type Update = ();
    async fn next(&mut self) -> Option<()> {
        self.source.latest = Some(Rc::new(self.source.source.next().await?));
        Some(())
    }
}
