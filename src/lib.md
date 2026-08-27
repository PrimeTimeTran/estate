//! # Prelude

//!
//! Estate is a workspace analysis and knowledge engine for understanding,
//! indexing, and navigating software projects.
//!
//! Estate builds a persistent model of a workspace by combining filesystem
//! discovery, source analysis, relationships, and resources into a unified
//! project graph.
//!
//! ## Capabilities
//!
//! Estate provides:
//!
//! - **Workspace discovery** — discovers files, directories, and projects.
//! - **Source analysis** — analyzes source code and extracts symbols and
//! relationships.
//! - **Project graphs** — models nodes and relationships between project
//! entities.
//! - **Indexing and search** — maintains indexes for efficient lookup.
//! - **Virtual filesystems** — provides filesystem access through the
//! [`vfs`] abstraction.
//! - **Wikilinks** — resolves Obsidian-style `[[Wikilinks]]` between resources.
//!
//! ## Architecture
//!
//! The primary entry point is [`EstateEngine`]. It coordinates Estate's
//! capabilities and provides a common interface for different frontends,
//! including the CLI, daemon, LSP, and future IDE integrations.
//!
//! The core domain model is represented by [`core`]. An [`Estate`] owns the
//! project's nodes, resources, relations, and bindings.
//!
//! Workspace discovery is handled by [`core::EstateDiscovery`], while
//! filesystem access is abstracted through [`vfs`]. The [`graph`] module
//! contains the project's graph and relationship logic.
//!
//! ## CLI
//!
//! The Estate CLI is provided by the `estate` binary.
//!
//! Install it from the repository root:
//!
//! `sh
//! cargo install --path . --bin estate
//! `
//!
//! Start the Estate daemon:
//!
//! `sh
//! estate daemon
//! `
//!
//! Run the daemon in the foreground with live logs:
//!
//! `sh
//! estate daemon --live
//! `
//!
//! Analyze a Rust project:
//!
//! `sh
//! estate analyze path/to/project
//! `
//!
//! ## Layout
//!
//! - [`core`] — Estate's domain model and workspace discovery.
//! - [`engine`] — high-level orchestration through [`EstateEngine`].
//! - [`graph`] — nodes, relationships, and graph operations.
//! - [`registry`] — persistent project and entity registration.
//! - [`vfs`] — virtual filesystem abstractions and implementations.
//! - [`daemon`] — long-running Estate daemon.
//! - [`router`] — routing between Estate operations and frontends.
//! - [`prelude`] — commonly used Estate types.
//!
//! Internal implementation modules are prefixed with `_` and are not intended
//! to form part of Estate's public API.
//!
//! ## Design Goals
//!
//! Estate is designed around a few principles:
//!
//! - **One project model** — frontends should consume the same Estate model
//! rather than implementing their own project representations.
//! - **Stable identities** — domain entities use UUID-based identities so
//! references remain stable across filesystem and application boundaries.
//! - **Backend independence** — filesystem access and persistence should be
//! replaceable without changing the domain model.
//! - **Frontend independence** — the CLI, daemon, LSP, and IDE integrations
//! should operate on the same underlying engine and domain abstractions.
#![allow(warnings)]

#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "mobile")]
pub mod mobile;

pub mod prelude;
pub mod share;
pub mod util;
