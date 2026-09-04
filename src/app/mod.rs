//! Application Business Logic:
//!
//! Generic traits should exist inside of ./src/app module
//! Platform implementations for native, mobile, web should exist in their own respective namespaces
//!
//! - [App]:
//! - [NativeApp]:
//! - [WebApp]:
//!
//! - ./src/native
//! - ./src/web
//! - ./src/mobile
//!
/// Generic traits should exist inside of ./src/app module
/// Platform implementations for native, mobile, web should exist in their own respective namespaces
///
/// ./src/native
/// ./src/web
/// ./src/mobile
//
pub mod app;
pub mod context;
pub mod event;
pub mod job;
pub mod model;
pub mod modules;
pub mod prelude;
pub mod state;
pub mod task;

/// Platform Gates
#[cfg(feature = "native")]
#[path = "./app.native.rs"]
pub mod app_native;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
#[path = "./app.web.rs"]
pub mod app_web;

#[cfg(not(all(feature = "web", target_arch = "wasm32")))]
#[path = "./app.web.stub.rs"]
pub mod app_web;

#[cfg(feature = "mobile")]
#[path = "./app.mobile.rs"]
pub mod app_mobile;

// Glob exports so we dont have to reimport everywhere.
pub use app::*;
pub use context::*;
pub use event::*;
pub use job::*;
// pub use prelude::*;
pub use state::*;
pub use task::*;
