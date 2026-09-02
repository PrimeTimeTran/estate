#![allow(warnings)]

#[cfg(feature = "web")]
pub mod web;

pub mod api;
pub mod app;
pub mod client;
pub mod data;
pub mod helpers;
pub mod model;
pub mod output;
pub mod prelude;
pub mod proto;
pub mod share;
pub mod tool;
pub mod ui;
pub mod util;
pub use tool::*;
pub use ui::*;

pub(crate) use crate::app::event as e;

// #[cfg(feature = "native")]
// pub mod event;
//
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
