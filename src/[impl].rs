//! ## [Impl]
//!
//! Kitchen draw placement for impls until placement has stabilized.
//!
use crate::{
	prelude::*,
	r#struct::{C, S},
};

impl<C> Clone for S<C> {
	fn clone(&self) -> Self {
		Self {
			context: PhantomData,
			state: PhantomData,
			view: self.view.clone(),
		}
	}
}
/// Manually implement Default specifically for S<C>
///
impl Default for S<C> {
	fn default() -> Self {
		S {
			view: ViewType::MarkdownScreen,
			context: PhantomData,
			state: PhantomData,
		}
	}
}

impl<C, S> structs::Renderer<C, S> {
	pub fn new(state: S, cancel: CancellationToken) -> Self {
		Self {
			cancel,
			state,
			phantom: PhantomData,
			view: ViewType::MarkdownScreen,
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			windows: vec![],
		}
	}
}

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
impl traits::Renderer for HostRenderer {
	#[cfg(target_arch = "wasm32")]
	fn render(&mut self) {
		// wasm rendering
	}
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn render(&mut self) {
		// native rendering
	}
}
