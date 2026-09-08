//! Application Business Logic:
//!
//! Generic traits should exist inside of ./src/app module
//! Platform implementations for native, mobile, web should exist in their own respective namespaces
//!
//! - [App]:
//! - [NativeApp]:
//! - [Clock]:
//! - [Provide]:
//!
//! - ./src/native
//! - ./src/web
//! - ./src/mobile
//!
/// Generic traits defined in ./src/trait module
/// Platform implementations for native, mobile, web should exist in their own respective namespaces
///
/// ./src/native
/// ./src/web
/// ./src/mobile
///
/// ./src/app/mod.rs
#[path = "app-entry.rs"]
pub mod app_entry;
#[path = "app-state.rs"]
pub mod app_state;
pub mod clock;
pub mod context;
pub mod event;
pub mod host;
pub mod job;
pub mod state;
pub mod task;
pub mod worker;

pub use self::{app_entry::*, clock::*, context::*, event::*, host::*, job::*, task::*, worker::*};

/// Platform Gates
#[cfg(feature = "native")]
#[path = "./native.app.rs"]
pub mod app_native;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
#[path = "./web.app.rs"]
pub mod app_web;

#[cfg(not(all(feature = "web", target_arch = "wasm32")))]
#[path = "./web.app.stub.rs"]
pub mod app_web;
