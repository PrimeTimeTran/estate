#![allow(warnings)]

pub mod api;
pub mod data;
pub mod helpers;
pub mod model;
pub mod prelude;
pub mod proto;
pub mod runtime;
pub mod services;
pub mod share;
pub mod tool;
pub mod r#trait;
pub mod ui;
pub mod util;

pub mod app;

pub use crate::app::event as e;
pub use crate::data::*;
pub use crate::runtime::*;
pub use crate::tool::*;
pub use crate::r#trait::*;
pub use crate::ui::{theme::*, *};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod server;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod native;
