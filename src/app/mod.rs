// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

// Available to all consumers, external and internal.
pub mod event;
pub mod model;
pub mod prelude;
pub mod state;

// Glob reexport to make it easier to use.
pub use app::*;
pub use context::*;
pub use event::*;
pub use job::*;
pub use state::*;
pub use task::*;

// Only inside of this crate.
pub(crate) mod app;
pub(crate) mod context;
pub(crate) mod job;
pub(crate) mod modules;
pub(crate) mod task;
pub(crate) use modules::*;
