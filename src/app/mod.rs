//! Application Business Logic:
//!
//! Generic traits should exist inside of ./src/app module
//! Platform implementations for native, mobile, web should exist in their own respective namespaces
//!
//! - [App]:
//! - [Clock]:
//! - [Provide]:
//!
#[path = "./task.rs"]
pub mod app_task;
/// Generic traits defined in ./src/trait module
/// Platform implementations for native, mobile, web should exist in their own respective namespaces
///
/// ./src/native
/// ./src/web
/// ./src/mobile
pub mod context;
pub mod host;
pub mod job;
pub mod worker;

#[path = "./[prelude].rs"]
pub mod app_prelude;

#[cfg(not(all(feature = "web", target_arch = "wasm32")))]
#[path = "./web.app.stub.rs"]
pub mod app_web;
