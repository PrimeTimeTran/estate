#![allow(warnings)]

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mobile")]
pub mod mobile;

pub mod app;

pub mod prelude;
pub mod share;
pub mod theme;
pub mod util;
pub mod ui;
