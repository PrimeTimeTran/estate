// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

// Publicly exposed
pub mod event;
pub mod model;
pub mod prelude;
pub mod state;

// Gated mods
//
#[cfg(feature = "native")]
#[path = "./app.native.rs"]
pub mod app_native;

#[cfg(feature = "web")]
#[path = "./app.web.rs"]
pub mod app_web;

#[cfg(feature = "mobile")]
#[path = "./app.mobile.rs"]
pub mod app_mobile;

// Glob exports for ease of use.
pub use app::*;
pub use context::*;
pub use event::*;
pub use job::*;
pub use state::*;
pub use task::*;

// Crate exposed only
pub(crate) mod app;
pub(crate) mod context;
pub(crate) mod job;
pub(crate) mod modules;
pub(crate) mod task;
pub(crate) use modules::*;
