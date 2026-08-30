// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

pub(crate) mod app;
pub(crate) mod context;
pub(crate) mod event;
pub(crate) mod host;
pub(crate) mod job;
pub(crate) mod model;
pub(crate) mod modules;
// pub mod servicesx;
pub(crate) mod session;
pub(crate) mod state;
pub(crate) mod task;

pub use app::*;
pub use context::*;
pub use event::*;
pub use job::*;
pub(crate) use modules::{
	// runtime::{Runtime, RuntimeState},
	*,
};
pub use session::*;
pub use task::*;

// #[cfg(not(target_arch = "wasm32"))]
// #[path = "../native/mod.rs"]
// pub(crate) mod native;

pub use anyhow::{Error, Result};
pub use serde::{Deserialize, Serialize};
pub use std::collections::{HashMap, HashSet, VecDeque};
