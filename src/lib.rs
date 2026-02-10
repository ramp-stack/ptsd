// #[derive(Debug, Component)]
// pub struct Listener<D: Drawable, T: Hash + Debug + Clone + 'static>(Stack, prism::Listener<D, T>);
// impl<D: Drawable, T: Hash + Debug + Clone + 'static> OnEvent for Listener<D, T> {}
// impl<D: Drawable, T: Hash + Debug + Clone + 'static> Listener<D, T> {
//     pub fn new(ctx: &mut Context, theme: &Theme, inner: D, updated_on: impl Fn(&mut Context, &Theme, &mut D, T) + 'static) -> Self {
//         let theme = theme.clone();
//         let updated_on = move |ctx: &mut Context, inner: &mut D, other: T| (updated_on)(ctx, &theme.clone(), inner, other);
//         Listener(Stack::default(), prism::Listener::new(ctx, inner, updated_on))
//     }
// }

pub mod theme;
pub use theme::*;
mod color;

pub mod interface;
pub use interface::*;

pub mod interactions;
pub mod utils;