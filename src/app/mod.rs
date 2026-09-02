// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

pub(crate) mod app;
pub(crate) mod context;
pub mod event;
pub(crate) mod host;
pub(crate) mod job;
pub(crate) mod model;
pub(crate) mod modules;
pub mod prelude;

pub(crate) mod state;
pub(crate) mod task;

pub use app::*;
pub use context::*;
pub use event::*;
pub use job::*;
pub(crate) use modules::*;

pub use task::*;

pub use anyhow::{Error, Result};
pub use serde::{Deserialize, Serialize};
pub use std::collections::{HashMap, HashSet, VecDeque};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod session;
#[cfg(not(target_arch = "wasm32"))]
pub use session::*;
