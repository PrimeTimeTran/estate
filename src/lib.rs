//! # Estate
//!
//! Estate is a workspace analysis and knowledge engine.
//!
//! It provides:
//!
//! - workspace and filesystem discovery
//! - symbol analysis
//! - dependency and relationship graphs
//! - indexing and search
//! - VFS support
//! - anchors and references
//!
//! ## Architecture
//!
//! The crate is organized around [`EstateEngine`], which exposes
//! Estate's capabilities to different frontends such as the CLI,
//! daemon, LSP, and future IDE applications.
//!
//! ## CLI
//!
//! Install the CLI:
//!
//! ```sh
//! cargo install --path . --bin estate
//! ```
//!
//! Then:
//!
//! ```sh
//! estate format path/to/file.rs
//! ```

#![allow(warnings)]
// #[allow(dead_code)]
// #[allow(unused_imports)] // Silences unused imports

// #![allow(unused_must_use)]
// #![allow(unused_variables)]

// Add warnings
// #![warn(dead_code)]
// #![warn(unused_mut)]
// #![warn(unused_parens)]
// #![warn(unused_braces)]
// #![warn(unused_must_use)]
// #![warn(unused_assignments)]
// #![warn(unused_imports)]
// #![warn(unused_variables)]

pub mod _core;
pub mod _shared;
pub mod _static;
pub mod constants;
pub mod core;
pub mod daemon;
pub mod engine;
pub mod prelude;
pub mod registry;
pub mod router;
pub mod vfs;
