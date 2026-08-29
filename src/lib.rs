#![allow(warnings)]

pub mod app;

pub mod data;
pub mod helpers;
pub mod prelude;
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

#[cfg(feature = "native")]
pub use crate::native::prelude::*;
