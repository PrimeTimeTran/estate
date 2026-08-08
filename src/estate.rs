use crate::{
	_core::*,
	daemon::daemon::{Action, ActionRegistry},
	vfs::Inode,
};
use anyhow::Error;
use std::path::Path;
use tower_lsp::jsonrpc::Response;
//                   ┌───────────────┐
//                   │ Estate Engine │
//                   │     (Rust)    │
//                   └───────┬───────┘
//                           │
//       ┌───────────────────┼───────────────────┐
//       │                   │                   │
//  Discovery             Registry             Graph
//       │                   │                   │
//  filesystem          resources/IDs       relationships
//  packages             aliases             deps
//  ignores              locations           parents
//  profiles             metadata            children
//       │                   │
//       └───────────┬───────┘
//                   │
//                Resolver
//                   │
//       ┌───────────┴───────────┐
//       │                       │
//    VS Code                   Zed
//     adapter                 adapter
///--------------------------------------------------------------------------------
/// Estate:
///
/// - install root personal estate
/// - init workspace/project
/// - init framework repo
/// - sync formatters(.prettierrc, .markdownlint.jsonc)
/// - sync .vscode/settings.json, .zed/settings.json
/// - build index/registry
///--------------------------------------------------------------------------------
// User types:
//     @my-pipeline
// Resolver:
//     @my-pipeline -> EstateId(55)
// Store:
//     EstateId(55) -> Resource
// Resource:
//     Location::File(".estate/pipelines/build.json")
// VFS:
//     open(file://...)
// Resolver = "what is this?"
// VFS      = "how do I access it?"
// Store    = "where do I remember it?"
// Graph    = "how is it related?"
// Bad
// vfs.get(id)
// vfs.get(path)
// vfs.get(alias)
// vfs.get(uri)
// vfs.get(wikilink)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EstateId(u64);

pub struct Estate {
	// pub discovery: DiscoveryEngine,
	// pub registry: EstateRegistry,
	// pub resolver: EstateResolver,
	// pub graph: EstateGraph,
	// pub vfs: EstateVfs,
	// pub config: EstateConfig,
	// pub cache: EstateCache,
}

pub struct Node;
pub struct NodeId;
trait Discovery {
	fn discover(&self, root: &Path) -> Result<DiscoveryResult, Error>;
}
pub trait Daemon {
	fn execute(&mut self, action: Action) -> Result<Response, Error>;
	fn start(&mut self) -> Result<(), Error>;
	fn stop(&mut self) -> Result<(), Error>;
}
/// Verb layer.
// format
// rename
// save
// index
// analyze
// build
// search
// resolve
// organize imports
// find references
// go to definition
pub struct EstateDaemon {
	pub estate: Estate,
	// pub vfs: EstateVfs,
	// pub graph: EstateGraph,
	pub actions: ActionRegistry,
	pub discovery: EstateDiscovery,
	// pub actions: ActionRegistry,
	// pub resolver: EstateResolver,
	// pub registry: EstateRegistry,
}
impl Daemon for EstateDaemon {
	fn execute(&mut self, _action: Action) -> Result<Response, Error> {
		todo!("")
	}
	fn start(&mut self) -> Result<(), Error> {
		todo!("")
	}
	fn stop(&mut self) -> Result<(), Error> {
		todo!("")
	}
}
pub trait Registry {
	fn get(&self, id: EstateId) -> Option<Resource>;
	fn upsert(&mut self, resource: Resource);
	fn remove(&mut self, id: EstateId);
}
pub trait Store {
	fn get(&self, id: EstateId) -> Option<Resource>;
	fn insert(&mut self, resource: Resource);
	fn update(&mut self, resource: Resource);
	fn remove(&mut self, id: EstateId);
}
pub trait Resolver {
	fn resolve(&self, id: EstateId) -> Option<Resource>;

	fn lookup(&self, reference: &str, scope: EstateScope) -> Vec<Resolution>;
}
pub trait EstateGraph {
	fn children(&self, id: EstateId) -> Vec<EstateId>;
	fn parents(&self, id: EstateId) -> Vec<EstateId>;
	fn dependencies(&self, id: EstateId) -> Vec<EstateId>;
}
pub trait Vfs {
	// fn open(&self, node: NodeId) -> Result<FileHandle, Error>;
	// fn stat(&self, node: NodeId) -> Result<Metadata, Error>;
	// fn watch(&self, node: NodeId) -> Result<WatchHandle, Error>;
	fn resolve_inode(&self, inode: Inode) -> Node;
	fn invalidate(&mut self, node: NodeId);
}
pub struct Resolution {
	pub id: EstateId,
	pub confidence: f32,
}

// pub trait EstateStore {
// 	fn get(&self, id: EstateId) -> Option<Resource>;
// 	fn find(&self, query: ResourceQuery) -> Vec<Resource>;
// 	fn put(&mut self, resource: Resource);

// 	/// Resolve a stable identity to a resource.
// 	fn resolve(&self, id: EstateId) -> Option<Resource>;

// 	/// Resolve a user-facing reference:
// 	/// path, alias, wikilink, symbol, anchor, etc.
// 	fn lookup(&self, reference: &str, scope: EstateScope) -> Vec<Resource>;

// 	/// Register or update a resource.
// 	fn upsert(&mut self, resource: Resource);

// 	/// Remove a resource.
// 	fn remove(&mut self, id: EstateId);

// 	/// Query children.
// 	fn children(&self, id: EstateId) -> Vec<Resource>;

// 	/// Get metadata.
// 	fn metadata(&self, id: EstateId) -> ResourceMetadata;
// }

/// LSP, Linter, FS Registry/index,
pub enum EstateScope {
	System,
	User,
	Workspace,
}

pub struct Resource {
	pub id: EstateId,
	pub kind: ResourceKind,
	pub locations: Vec<Location>,
	pub aliases: Vec<String>,
}
pub enum ResourceKind {
	File,
	Symbol,
	Anchor,
	Workspace,
	Project,
	Generated,
}
pub struct ResourceMetadata {
	pub created_at: u64,
	pub updated_at: u64,
	pub scope: EstateScope,
}
