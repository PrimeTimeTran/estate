//! Wrapper struct for core components & services used by the Estate Engine
//!
//! # Description
//!
use crate::api::Api;

pub use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct EstateEngine<R: Runtime> {
	// Domain
	pub estate: Estate,
	pub runtime: Arc<R>,

	// Infrastructure
	pub vfs: EstateVfs,
	pub index: EstateIndex,
	// pub workspace: Workspace,

	// pub index: OnceCell<EstateIndex>,
	// pub search: OnceCell<SearchService>,
	// pub analysis: OnceCell<AnalysisService>,

	// domain representation
	pub graph: EstateGraph,

	// persistence/coordination
	pub registry: EstateRegistry,

	/// Capabilities
	pub resolver: EstateResolver,
	pub discovery: EstateDiscovery,
	pub anchors: AnchorService,
	pub search: SearchService,
	pub analysis: AnalysisService,
}
impl<R: Runtime> EstateEngine<R> {
	pub fn new(runtime: R) -> Result<Self> {
		// let state = EstateState::load_from_disk().unwrap();
		// let state_monitor = StateMonitor::new(&state_path)?;
		Ok(Self {
			runtime: Arc::new(runtime),
			estate: Estate::default(),
			registry: EstateRegistry::default(),
			index: EstateIndex::default(),
			resolver: EstateResolver::default(),
			graph: EstateGraph::default(),
			discovery: EstateDiscovery::default(),
			vfs: EstateVfs::default(),
			anchors: AnchorService::default(),
			search: SearchService::default(),
			analysis: AnalysisService::default(),
		})
	}
	pub fn session(&mut self) -> Session {
		self.runtime.session()
	}
	pub fn runtime(&self) -> Arc<R> {
		Arc::clone(&self.runtime)
	}
	#[cfg(not(feature = "web"))]
	pub async fn format(self, args: &FormatArgs) -> Result<String, Error> {
		daemon::LintDaemon.run(&args).await;
		Ok("Success".to_string())
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateRegistry;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateIndex;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateResolver;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateGraph;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateVfs;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct AnchorService {
	registry: EstateRegistry,
	index: EstateIndex,
	resolver: EstateResolver,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct SearchService;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct AnalysisService;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ReferenceKind {
	#[default]
	File,
	Link,
	Embed,
	Relative,
	Anchor,
	Asset,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Reference<'a> {
	pub target: &'a str,
	pub fragment: Option<&'a str>,
	pub kind: ReferenceKind,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ResolveContext {
	pub scope: EstateScope,
	pub from: Uuid,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum EstateScope {
	System,
	User,
	#[default]
	Workspace,
}

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

use crate::prelude::*;

// Anchor/bookmark store...? FS store...? Asset store?
// - "I need a thing, give it to me"
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
use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Change {
	Created(Uuid),
	Modified(Uuid),
	Deleted(Uuid),
	Renamed { from: Uuid, to: Uuid },
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
