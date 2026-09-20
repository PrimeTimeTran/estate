//! # Estate
//!
//! A cross platform/framework "global" workspace the merges package configs across multiple shells, IDEs, & project types.
//!
//! Think of the Estate system as
//! - "Obsidian with a Code Editor"
//! - A cross framework workspace.
//!
//! ## Goal
//!
//! Reduce mental fatigue such as recalling where you defined
//! "that one bash script".
//!
//! ### Inspiration:
//!
//! - [Dot Repo]:
//! - [Obsidian Wikilinks]:
//!
//! - [App]: Entrypoint
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

pub mod app;
#[path = "./[app].rs"]
pub mod app_entry;
pub mod data;
pub mod helper;
#[path = "./[impl].rs"]
pub mod impls;
#[path = "./[macro].rs"]
pub mod macros;
pub mod model;
pub mod modules;
pub mod prelude;
pub mod proto;
pub mod service;
pub mod share;
#[path = "./[struct].rs"]
pub mod structs;
#[path = "./[trait].rs"]
pub mod traits;
pub mod ui;
pub mod util;

pub use crate::{app::app_prelude, data::*, modules::*, service::event as e, ui::*};
