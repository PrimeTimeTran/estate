#![allow(warnings)]
pub mod app;
pub(crate) use crate::app::event as e;

pub use e::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use crate::client::*;

pub mod helpers;
pub mod prelude;

pub mod share;
pub mod theme;
pub mod tool;
pub mod ui;
pub use ui::*;
pub mod util;

pub mod data;
pub(crate) use crate::data::default;

// A 'central' native gate doesn't work.
// #[cfg(feature = "native")]
// pub mod lib_native;
// #[cfg(feature = "native")]
// pub use lib_native::*;

pub mod model;
pub use model::*;

pub mod proto;
pub use proto::{leetcode::types::*, *};

// #[cfg(feature = "native")]
pub mod services;
// #[cfg(feature = "native")]
pub use services::*;

#[cfg(feature = "native")]
pub mod native;
#[cfg(feature = "native")]
use native::*;

// #[cfg(feature = "native")]
// pub mod data;

#[cfg(feature = "native")]
pub mod event;
#[cfg(feature = "native")]
pub use event::*;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mobile")]
pub mod mobile;
