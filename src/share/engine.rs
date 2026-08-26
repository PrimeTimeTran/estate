//! Wrapper struct for core components & services used by the Estate Engine
//!
//! # Description
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

use crate::{ app::{ model::*, modules::runtime::Runtime }, prelude::* };
use revelation::analyzer::Workspace;

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
	fn get(&self, id: Uuid) -> Option<Resource>;
	fn upsert(&mut self, resource: Resource);
	fn remove(&mut self, id: Uuid);
}
/// Index = derived structure optimized for finding that knowledge.
pub trait Index {
	fn generation(&self) -> u64;
	fn lookup(&self, query: &Query) -> Vec<Uuid>;
	fn invalidate(&mut self, change: &Change);
}
/// I have foo but I want bar
/// - Inline IDE Anchor -> FS file for preview
/// - Inline wikilink -> FS asset for embed
/// Resolver
///     "What does C mean?"
pub trait Resolver {
	fn resolve(&self, reference: &Reference, context: &ResolveContext) -> Vec<Resolution>;
}

/// Initialization
/// - I installed estate engine
/// - I opened the estate IDE
// pub trait Discovery {
// 	fn discover(&self, root: &Path) -> Result<DiscoveryResult, Error>;
// }
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
// 	fn get(&self, id: Uuid) -> Option<Resource>;
// 	fn insert(&mut self, resource: Resource);
// 	fn update(&mut self, resource: Resource);
// 	fn remove(&mut self, id: Uuid);
// }

/// Abstraction for ranking responses which are not deteminitic.
/// - "Give me package.json" can produce many results
/// - "Give me available" commnands can produce different results depending on file .ext, settings.json, UI focus, and state.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Resolution {
	pub id: Uuid,
	pub confidence: f32,
	pub resource: Resource,
	pub fragment: Option<String>,
	// pub reason: ResolutionReason,
}

///--------------------------------------------------------------------------------
/// Estate:
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
//     @my-pipeline -> Uuid(55)
// Store:
//     Uuid(55) -> Resource
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
// 	fn get(&self, id: Uuid) -> Option<Resource>;
// 	fn find(&self, query: ResourceQuery) -> Vec<Resource>;
// 	fn put(&mut self, resource: Resource);

// 	/// Resolve a stable identity to a resource.
// 	fn resolve(&self, id: Uuid) -> Option<Resource>;

// 	/// Resolve a user-facing reference:
// 	/// path, alias, wikilink, symbol, anchor, etc.
// 	fn lookup(&self, reference: &str, scope: EstateScope) -> Vec<Resource>;

// 	/// Register or update a resource.
// 	fn upsert(&mut self, resource: Resource);

// 	/// Remove a resource.
// 	fn remove(&mut self, id: Uuid);

// 	/// Query children.
// 	fn children(&self, id: Uuid) -> Vec<Resource>;

// 	/// Get metadata.
// 	fn metadata(&self, id: Uuid) -> ResourceMetadata;
// }

/// LSP, Linter, FS Registry/index,

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Change {
	Created(Uuid),
	Modified(Uuid),
	Deleted(Uuid),
	Renamed {
		from: Uuid,
		to: Uuid,
	},
	ConfigChanged,
	WorkspaceChanged,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Query {
	pub text: String,
	pub scope: EstateScope,
	pub context: ResolutionContext,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ResolutionContext;
