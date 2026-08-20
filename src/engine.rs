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
	fn resolve(&self, reference: &Reference, context: &ResolveContext) -> Vec<Resolution>;
}

/// Initialization
/// - I installed estate engine
/// - I opened the estate IDE
pub trait Discovery {
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
/// Abstraction for ranking responses which are not deteminitic.
/// - "Give me package.json" can produce many results
/// - "Give me available" commnands can produce different results depending on file .ext, settings.json, UI focus, and state.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Resolution {
	pub id: EstateId,
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum EstateScope {
	System,
	User,
	#[default]
	Workspace,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct EstateId(u64);
static NEXT_ESTATE_ID: AtomicU64 = AtomicU64::new(1);
impl EstateId {
	pub fn new() -> Self {
		Self(NEXT_ESTATE_ID.fetch_add(1, Ordering::Relaxed))
	}
}
/// Estate owns the capabilities and domain model; the daemon exposes those capabilities as a long-lived service.
#[derive(Clone, Debug, Hash)]
pub struct EstateEngine {
	pub estate: Estate,       // domain model
	pub vfs: EstateVfs,       // infrastructure
	pub index: EstateIndex,   // infrastructure
	pub workspace: Workspace, // domain/context

	// pub index: OnceCell<EstateIndex>,
	// pub search: OnceCell<SearchService>,
	// pub analysis: OnceCell<AnalysisService>,
	pub graph: EstateGraph,         // domain representation
	pub registry: EstateRegistry,   // persistence/coordination
	pub resolver: EstateResolver,   // capability
	pub discovery: EstateDiscovery, // capability
	pub anchors: AnchorService,     // capability
	pub search: SearchService,      // capability
	pub analysis: AnalysisService,  // capability
}
impl EstateEngine {
	pub fn new() -> anyhow::Result<Self> {
		Ok(Self {
			estate: Estate::default(),
			workspace: Workspace::new(),
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
	pub async fn format(self, args: &FormatArgs) -> anyhow::Result<String, anyhow::Error> {
		LintDaemon.run(&args).await;
		Ok("Success".to_string())
	}
}

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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct EstateRegistry;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct EstateIndex;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct EstateResolver;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct EstateGraph;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct EstateDiscovery;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct EstateVfs;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct AnchorService {
	registry: EstateRegistry,
	index: EstateIndex,
	resolver: EstateResolver,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct SearchService;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
struct AnalysisService;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ReferenceKind {
	#[default]
	File,
	Link,
	Embed,
	Relative,
	// potentially:
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
	pub from: EstateId,
}
