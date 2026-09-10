//! # Estate
//! 
//! Idea is a "global workspace" the merges package configs across multiple shells, IDEs, & project types.
//!
//! ## Goal
//! The goal is to reduce the mental fatigue experienced when trying to recall where "that one script" was.
//! Some of the ideas that estate borrows from is [`dot repo`], stuff[^1]
//!
//! Obsidian with a code editor.
//!
//! - [App]: Core business logic
//! - [AppRuntime]: Platform specific capabilities.
//! - [EstateEngine]: Estate Paradigm business logic utilities including vfs, resolvers,
//!
//! ## Build Targets
//!
//! - [Laptop/Desktop](NativeApp)
//! - [Web/Browser](WebApp)
//!
// #![allow(warnings)]

// #![feature(associated_type_defaults)]

// Disables unused input variables
// #![allow(unused_variables)]
// Disables unused input variables
// #![allow(unused_results)]

pub mod api;
pub mod app;
pub mod data;
pub mod helpers;
#[path = "./[impl].rs"]
pub mod impls;
pub mod r#macro;
pub mod model;
pub mod prelude;
pub mod proto;
pub mod runtime;
pub mod services;
pub mod share;
#[path = "./[struct].rs"]
pub mod structs;
pub mod tool;
#[path = "./[trait].rs"]
pub mod traits;
pub mod ui;
pub mod util;

pub use crate::app::app_prelude;
pub use crate::app::event as e;
pub use crate::r#macro as app_macros;
pub use crate::ui::{ui_prelude::*, *};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod native;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use crate::native::state as native_state;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod server;
