// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

pub use std::{
	collections::{HashMap, HashSet, VecDeque},
	// May not work on WASM?
	// path::Path,
};

pub use anyhow::{Error, Result};

pub mod prelude;
pub use crate::app::prelude::*;

pub(crate) mod app;
pub(crate) mod context;
pub(crate) mod event;
pub(crate) mod event_channel;
pub(crate) mod host;
pub(crate) mod job;
pub(crate) mod model;
pub(crate) mod modules;
pub(crate) mod state;
pub(crate) mod ve;
pub use context::*;

#[cfg(feature = "../native/mod.rs")]
pub(crate) mod native;
