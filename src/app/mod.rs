//! Application Business Logic:
//!
//! Generic traits should exist inside of ./src/app module
//! Platform implementations for native, mobile, web should exist in their own respective namespaces
//!
//! - [App]:
//! - [Clock]:
//! - [Provide]:
//!
/// Generic traits defined in ./src/trait module
/// Platform implementations for native, mobile, web should exist in their own respective namespaces
///
/// ./src/native
/// ./src/web
/// ./src/mobile
pub mod context;
pub mod event;
pub mod host;
pub mod job;
pub mod task;
pub mod worker;

#[path = "./[prelude].rs"]
pub mod app_prelude;

/// Platform Gates
#[cfg(feature = "native")]
#[path = "./native.app.rs"]
pub mod app_native;

#[cfg(not(all(feature = "web", target_arch = "wasm32")))]
#[path = "./web.app.stub.rs"]
pub mod app_web;
