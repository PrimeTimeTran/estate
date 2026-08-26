// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

pub use std::collections::{ HashMap, HashSet, VecDeque };

pub mod prelude;
pub use crate::app::prelude::*;

pub(crate) mod host;
pub(crate) mod job;
pub(crate) mod model;
pub(crate) mod modules;
pub(crate) mod app;
pub(crate) mod state;
pub(crate) mod event;
#[cfg(feature = "native")]
pub(crate) mod context;
#[cfg(feature = "native")]
pub use context::*;

#[cfg(feature = "native")]
pub(crate) mod state_native;

#[path = "modules/monitor.rs"]
pub(crate) mod monitor;
#[cfg(not(target_arch = "wasm32"))]
#[path = "modules/monitor_native.rs"]
pub(crate) mod monitor_native;
