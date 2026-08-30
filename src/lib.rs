#![allow(warnings)]

pub mod app;
pub mod data;
pub mod helpers;
pub mod prelude;
pub mod proto;
pub mod share;
pub mod theme;
pub mod tool;
pub mod ui;
pub mod util;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mobile")]
pub mod mobile;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub use native::*;

#[cfg(feature = "native")]
#[path = "./event/mod.rs"]
pub mod event;
pub use event::*;

#[path = "./event/handler.rs"]
pub mod handler;
pub use handler::*;

#[path = "data/native.rs"]
pub mod native_data;
