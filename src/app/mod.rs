use crate::{ share::{ prelude::* } };

pub(crate) mod modules;
pub(crate) mod model;

pub use modules::runtime::Runtime;

// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/mobile
// ./src/web

pub struct App<R: Runtime> {
	pub engine: model::EstateEngine<R>,
}
impl<R: Runtime> App<R> {
	pub fn new(engine: model::EstateEngine<R>) -> Self {
		Self { engine }
	}
}
impl<R: Runtime> App<R> {}
