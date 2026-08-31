#![allow(warnings)]
pub mod app;

#[path = "./app/event.rs"]
pub(crate) mod app_event;
pub use app_event::*;

pub mod helpers;
pub mod prelude;
pub mod proto;

pub mod share;
pub mod theme;
pub mod tool;
pub mod ui;
use ui::*;
pub mod util;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mobile")]
pub mod mobile;

// A 'central' native gate doesn't work.
// #[cfg(feature = "native")]
// pub mod lib_native;
// #[cfg(feature = "native")]
// pub use lib_native::*;
//
#[cfg(feature = "native")]
pub mod services;
#[cfg(feature = "native")]
pub use services::*;

#[cfg(feature = "native")]
pub mod native;
#[cfg(feature = "native")]
use native::*;

#[cfg(feature = "native")]
pub mod data;

#[cfg(feature = "web")]
#[path = "./data/default.rs"]
pub mod data;

// So thats why we do it individually here.
#[cfg(feature = "native")]
pub mod event;
#[cfg(feature = "native")]
pub use event::*;
