#![allow(warnings)]
pub mod app;
pub(crate) use crate::app::event as e;

pub mod data;
pub mod helpers;
pub mod model;
pub mod output;
pub mod prelude;
pub mod proto;
pub mod services;
pub mod share;
pub mod tool;
pub use tool::*;
pub mod util;

pub mod ui;
pub use ui::*;

#[cfg(feature = "native")]
pub mod event;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mobile")]
pub mod mobile;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use crate::native::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod client;

#[cfg(not(target_arch = "wasm32"))]
pub use services::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_lib {
	pub use crate::native::*;
}
