use crate::{
	_core::*,
	daemon::{
		daemon::{Action, ActionRegistry},
		start::{ActionRequest, DaemonError, DaemonOptions},
	},
	vfs::Inode,
};
use anyhow::Error;
use async_trait::async_trait;
use std::path::Path;
use tower_lsp::jsonrpc::Response;

//                        ┌─────────────────────┐
//                        │       Estate        │
//                        │  semantic engine    │
//                        └──────────┬──────────┘
//                                   │
//      ┌──────────────┬─────────────┼──────────────┬─────────────┐
//      ▼              ▼             ▼              ▼             ▼
// Discovery       Registry        Index         Graph          VFS
//      │              │             │              │             │
// "what exists?"  "what is it?" "find it fast" "how related" "access it"
//      │              │             │              │             │
//      └──────────────┴─────────────┴──────────────┴─────────────┘
//                                   │
//                               Resolver
//                                   │
//                           "what did they mean?"
//                                   │
//                                   ▼
//                               Actions
//                                   │
//                                   ▼
//                               Daemon
//                                   │
//                        ┌──────────┼──────────┐
//                        ▼          ▼          ▼
//                      Zed       VS Code      CLI

pub trait Engine {
	// IDE anchors/bookmarks
	fn upsert() -> Result<(), Error>;
	fn read() -> Result<(), Error>;
	fn delete() -> Result<(), Error>;
	// .estate workspace (initial personal and then public/repo)
	// fn upsert() -> Result<(), Error>;
	// fn read() -> Result<(), Error>;
	// fn delete() -> Result<(), Error>;

	// CRUD .estate registry for IDE/discovery services
	// fn upsert() -> Result<(), Error>;
	// fn read() -> Result<(), Error>;
	// fn delete() -> Result<(), Error>;

	// CRUD .estate index for IDE/discovery services
	// fn upsert() -> Result<(), Error>;
	// fn read() -> Result<(), Error>;
	// fn delete() -> Result<(), Error>;
}
/// Registry = authoritative knowledge.
pub trait Registry {
	fn get(&self, id: EstateId) -> Option<Resource>;
	fn upsert(&mut self, resource: Resource);
	fn remove(&mut self, id: EstateId);
}
/// Index = derived structure optimized for finding that knowledge.
pub trait Index {
	fn generation(&self) -> u64;
	fn lookup(&self, query: &Query) -> Vec<EstateId>;
	fn invalidate(&mut self, change: &Change);
}
/// I have foo but I want bar
/// - Inline IDE Anchor -> FS file for preview
/// - Inline wikilink -> FS asset for embed
/// Resolver
///     "What does C mean?"
pub trait Resolver {
	fn resolve(&self, id: EstateId) -> Option<Resource>;
	fn lookup(&self, reference: &str, scope: EstateScope) -> Vec<Resolution>;
}
/// Initialization
/// - I installed estate engine
/// - I opened the estate IDE
trait Discovery {
	fn discover(&self, root: &Path) -> Result<DiscoveryResult, Error>;
}
// #[async_trait]
// pub trait Daemon {
//     async fn execute(
//         &mut self,
//         action: ActionRequest,
//     ) -> Result<Response, Error>;
//     async fn start(
//         &mut self,
//         options: Option<DaemonOptions>,
//     ) -> Result<(), Error>;
//     async fn stop(&mut self) -> Result<(), Error>;
// }
/// Anchor/bookmark store...? FS store...? Asset store?
/// - "I need a thing, give it to me"
// pub trait Store {
// 	fn get(&self, id: EstateId) -> Option<Resource>;
// 	fn insert(&mut self, resource: Resource);
// 	fn update(&mut self, resource: Resource);
// 	fn remove(&mut self, id: EstateId);
// }
/// Graph
///     "What is A connected to?"
/// - I created an estate .md file which wikilinks to 5 other estate files. Do I do a full table scan of the registry every time? No, the resolver should take in an estate id and context and give me back what it is I'm looking for. If I've opened the IDE from a repo/workspace then the link will look differently to resolve.
pub trait Graph {
	fn children(&self, id: EstateId) -> Vec<EstateId>;
	fn parents(&self, id: EstateId) -> Vec<EstateId>;
	fn dependencies(&self, id: EstateId) -> Vec<EstateId>;
}
/// Handles Disk operations and virtual one day (if a web version were needed)
pub trait Vfs {
	// fn open(&self, node: NodeId) -> Result<FileHandle, Error>;
	// fn stat(&self, node: NodeId) -> Result<Metadata, Error>;
	// fn watch(&self, node: NodeId) -> Result<WatchHandle, Error>;
	fn resolve_inode(&self, inode: Inode) -> Node;
	fn invalidate(&mut self, node: NodeId);
}
/// Abstraction for ranking responses which are not deteminitic.
/// - "Give me package.json" can produce many results
/// - "Give me available" commnands can produce different results depending on file .ext, settings.json, UI focus, and state.
pub struct Resolution {
	pub id: EstateId,
	pub confidence: f32,
	// pub reason: ResolutionReason,
}

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
//
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EstateId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId;

/// Estate owns the capabilities and domain model; the daemon exposes those capabilities as a long-lived service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Estate {
	pub registry: EstateRegistry,
	pub index: EstateIndex,
	pub resolver: EstateResolver,
	pub graph: EstateGraph,
	pub discovery: EstateDiscovery,
	pub vfs: EstateVfs,
	pub anchors: AnchorService,
	pub search: SearchService,
	pub analysis: AnalysisService,
}

/// Verb layer: format, rename, save, index, analyze, build, search, resolve, organize imports, find references, go to definition
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
// impl Daemon for EstateDaemon {
// 	fn execute(&mut self, _action: Action) -> Result<Response, Error> {
// 		todo!("")
// 	}
// 	fn start(&mut self) -> Result<(), Error> {
// 		todo!("")
// 	}
// 	fn stop(&mut self) -> Result<(), Error> {
// 		todo!("")
// 	}
// }

pub enum Change {
	Created(NodeId),
	Modified(NodeId),
	Deleted(NodeId),
	Renamed { from: NodeId, to: NodeId },
	ConfigChanged,
	WorkspaceChanged,
}
pub struct Query {
	pub text: String,
	pub scope: EstateScope,
	pub context: ResolutionContext,
}
struct ResolutionContext;

/// Data / truth
/// - Registry
/// Derived infrastructure
/// - Index
/// - Graph
/// - Cache
/// Domain behavior
/// - AnchorService
/// - SearchService
/// - AnalysisService
/// - Formatter
/// Runtime / infrastructure
/// - Daemon
/// - VFS
/// - logging
/// - tasks
/// - IPC
/// - watchers
/// - retry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EstateRegistry;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EstateIndex;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EstateResolver;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EstateGraph;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EstateDiscovery;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EstateVfs;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnchorService {
	registry: EstateRegistry,
	index: EstateIndex,
	resolver: EstateResolver,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SearchService;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AnalysisService;
