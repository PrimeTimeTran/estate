#![allow(warnings)]

pub mod api;
pub mod data;
pub mod helpers;
pub mod model;
pub mod prelude;
pub mod proto;
pub mod services;
pub mod share;
pub mod tool;
pub mod r#trait;
pub mod ui;
pub mod util;

pub use tool::*;
pub use r#trait::*;
pub use ui::*;

pub mod app;
pub use crate::app::event as e;

#[cfg(not(all(feature = "web", target_arch = "wasm32")))]
pub mod server;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web;

#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "native")]
pub mod server;

// pub mod event;
//
// pub mod services;
//
// #[cfg(feature = "mobile")]
// pub mod mobile;
//
// #[cfg(not(target_arch = "wasm32"))]
// pub mod native;
// #[cfg(not(target_arch = "wasm32"))]
// pub(crate) use crate::native::*;
//
// #[cfg(not(target_arch = "wasm32"))]
// pub mod client;
//
// #[cfg(not(target_arch = "wasm32"))]
// pub use services::*;
//
// #[cfg(not(target_arch = "wasm32"))]
// pub mod native_lib {
// pub use crate::native::*;
// }
//
