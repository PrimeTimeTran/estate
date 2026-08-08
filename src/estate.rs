use crate::daemon::daemon::ActionRegistry;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
pub type EstateId = u64;
pub struct Resource {
	pub id: EstateId,
	pub kind: ResourceKind,
	pub locations: Vec<Location>,
	pub aliases: Vec<String>,
}
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
// trait Discovery {
// 	fn discover(&self, root: &Path) -> Result<DiscoveryResult>;
// }
// struct DiscoveryResult {
// 	workspace: Workspace,
// 	packages: Vec<Package>,
// 	files: Vec<PathBuf>,
// 	ignored: Vec<PathBuf>,
// }
trait Registry {
	fn get(&self, id: EstateId) -> Option<Resource>;
	fn upsert(&mut self, resource: Resource);
	fn remove(&mut self, id: EstateId);
}
pub trait EstateStore {
	fn get(&self, id: EstateId) -> Option<Resource>;
	fn insert(&mut self, resource: Resource);
	fn update(&mut self, resource: Resource);
	fn remove(&mut self, id: EstateId);
}
trait Resolver {
	fn resolve_reference(&self, reference: &str) -> Vec<Resource>;
}
pub trait EstateResolver {
	fn resolve(&self, id: EstateId) -> Option<Resource>;
	fn lookup(&self, reference: &str, scope: EstateScope) -> Vec<Resolution>;
}
pub struct Resolution {
	pub resource: Resource,
	pub confidence: f32,
}
pub trait EstateGraph {
	fn children(&self, id: EstateId) -> Vec<EstateId>;
	fn parents(&self, id: EstateId) -> Vec<EstateId>;
	fn dependencies(&self, id: EstateId) -> Vec<EstateId>;
}
// trait Vfs {
// 	// fn open(uri: Uri) -> FileHandle;
// 	// fn stat(uri: Uri) -> Metadata;
// 	// fn watch(uri: Uri);
// 	fn resolve_inode(&self, inode: Inode) -> Node;
// 	fn dependencies(&self, node: NodeId) -> Vec<NodeId>;
// 	fn invalidate(&mut self, node: NodeId);
// }
// use crate::vfs::vfs::*;
struct Node;
struct NodeId;
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

pub struct EstateDaemon {
	pub actions: ActionRegistry,
	// pub fs: Box<dyn EstateFS>,
}

pub enum EstateScope {
	System,
	User,
	Workspace,
}
///--------------------------------------------------------------------------------
/// Profiles
///--------------------------------------------------------------------------------
// estate discover --profile rust-workspace
// estate discover --profile personal
// estate doctor
// pub const RUST_WORKSPACE: DiscoveryProfile = DiscoveryProfile {
//     name: "rust-workspace",
//     probes: PROBES_RUST_ZED,
// };
// pub const PERSONAL_ESTATE: DiscoveryProfile = DiscoveryProfile {
//     name: "personal-estate",
//     probes: PROBES_PERSONAL,
// };
///--------------------------------------------------------------------------------
/// Project/FS/Indexer/
///--------------------------------------------------------------------------------
pub struct EstateDiscovery {
	pub store: DiscoveryStore,
	// pub tasks: DiscoveryStore,
	pub task_tx: mpsc::Sender<Task>,
}
impl EstateDiscovery {
	// pub fn init(probes: ProbeSet) -> std::io::Result<DiscoveryStore> {
	// [*] Hard coded until discovery is stabilizes
	pub fn init() -> std::io::Result<DiscoveryStore> {
		let cwd = std::env::current_dir()?;
		let mut discovery = EstateDiscovery {
			store: DiscoveryStore::default(),
			task_tx: Self::prepare(),
		};
		walk_root_to_path(cwd, |dir| {
			println!("dir {:?}", dir);
			// for probe in probes {
			for probe in PROBES_PERSONAL {
				if let Some(path) = Self::probe(dir, probe) {
					discovery.emit(DiscoveryEvent::Found(RawDiscovery { probe, path }));
				}
			}
			WalkControl::Continue
		})?;
		Ok(discovery.store)
	}
	// Creates a background task pipeline.
	//
	// Returns the sender side (`tx`) so producers can submit work.
	// Spawns a worker that owns the receiver side (`rx`) and processes
	// tasks independently.
	//
	// Flow:
	//
	//     EstateDiscovery
	//          |
	//          |  send(Task)
	//          v
	//     mpsc channel (buffer: 100)
	//          |
	//          v
	//     worker(rx)
	//          |
	//          +--> index files
	//          +--> generate configs
	//          +--> run background jobs
	//
	// Multiple producers can clone the sender and enqueue work.
	// The worker runs concurrently and consumes tasks as they arrive.
	pub fn prepare() -> mpsc::Sender<Task> {
		let (tx, rx) = mpsc::channel::<Task>(100);
		tokio::spawn(worker(rx));
		tx
	}
	pub fn probe(dir: &Path, probe: &Probe) -> Option<PathBuf> {
		let path = dir.join(probe.name);
		let found = match probe.kind {
			ProbeKind::Directory => path.is_dir(),
			ProbeKind::File => path.is_file(),
		};
		found.then_some(path)
	}
}
impl EstateDiscovery {
	fn resolve(raw: RawDiscovery) -> Option<DiscoveryItem> {
		match raw.probe.id {
			"estate" => Some(DiscoveryItem::Estate(raw.path)),
			"git" => Some(DiscoveryItem::GitRepo(raw.path)),
			"cargo" => Some(DiscoveryItem::CargoProject(raw.path)),
			"zed" => Some(DiscoveryItem::Editor(EditorKind::Zed)),
			_ => None,
		}
	}
}
#[derive(Debug)]
pub struct RawDiscovery {
	pub probe: &'static Probe,
	pub path: PathBuf,
}
#[async_trait]
impl DiscoverySink for EstateDiscovery {
	async fn emit(&mut self, event: DiscoveryEvent) {
		match event {
			DiscoveryEvent::Found(raw) => {
				if let Some(item) = Self::resolve(raw) {
					self.store.items.push(item);
				}
			}
			DiscoveryEvent::StartTask(task) => {
				self.task_tx.send(task).await.unwrap();
			}
		}
	}
}
#[derive(Debug)]
pub enum Tool {}
#[derive(Debug, Default)]
pub struct DiscoveryStore {
	pub items: Vec<DiscoveryItem>,
}
#[derive(Debug)]
pub enum DiscoveryEvent {
	Found(RawDiscovery),
	StartTask(Task),
}
#[derive(Debug)]
pub enum EditorKind {
	Zed,
	NeoVim,
	Lapce,
	VSCode,
}
#[derive(Debug)]
pub enum DiscoveryItem {
	Estate(PathBuf),
	GitRepo(PathBuf),
	CargoProject(PathBuf),
	Editor(EditorKind),
}
#[async_trait]
pub trait DiscoverySink {
	async fn emit(&mut self, event: DiscoveryEvent);
}
pub enum WalkControl {
	Continue,
	Stop,
}
pub fn walk_root_to_path<F>(target: impl AsRef<Path>, mut visit: F) -> std::io::Result<WalkControl>
where
	F: FnMut(&Path) -> WalkControl,
{
	let target = target.as_ref().canonicalize()?;
	let mut current = filesystem_root(&target);
	if let WalkControl::Stop = visit(&current) {
		return Ok(WalkControl::Stop);
	}
	for component in target.strip_prefix(&current).unwrap().components() {
		current.push(component);
		if let WalkControl::Stop = visit(&current) {
			return Ok(WalkControl::Stop);
		}
	}
	Ok(WalkControl::Continue)
}
fn filesystem_root(path: &Path) -> PathBuf {
	PathBuf::from(path.components().next().unwrap().as_os_str())
}
#[derive(Debug, Clone, Copy)]
pub enum ProbeKind {
	File,
	Directory,
}
#[derive(Debug, Clone, Copy)]
pub struct Probe {
	pub id: &'static str,
	pub name: &'static str,
	pub kind: ProbeKind,
}
pub type ProbeSet = &'static [Probe];
pub const PROBES_MINIMAL: ProbeSet = &[
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
];
pub const PROBES_NODE: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "vscode",
		name: ".vscode",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "npm",
		name: "package.json",
		kind: ProbeKind::File,
	},
	Probe {
		id: "lockfile",
		name: "package-lock.json",
		kind: ProbeKind::File,
	},
	Probe {
		id: "prettier",
		name: ".prettierrc",
		kind: ProbeKind::File,
	},
];
pub const PROBES_RUST_ZED: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "zed",
		name: ".zed",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
	Probe {
		id: "rust_lock",
		name: "Cargo.lock",
		kind: ProbeKind::File,
	},
];
pub const PROBES_PERSONAL: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "bookmarks",
		name: "bookmarks",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "knowledge",
		name: "knowledge",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "commands",
		name: "commands",
		kind: ProbeKind::Directory,
	},
];
pub const REACT_CONFIGS: ProbeSet = &[
	Probe {
		id: "js",
		name: "next.config.js",
		kind: ProbeKind::File,
	},
	Probe {
		id: "ts",
		name: "next.config.ts",
		kind: ProbeKind::File,
	},
];
pub const PROBES_MONOREPO: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "workspace",
		name: "pnpm-workspace.yaml",
		kind: ProbeKind::File,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
	Probe {
		id: "npm",
		name: "package.json",
		kind: ProbeKind::File,
	},
];
pub const PROBES: ProbeSet = &[
	Probe {
		id: "estate",
		name: ".estate",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "git",
		name: ".git",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "vscode",
		name: ".vscode",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "zed",
		name: ".zed",
		kind: ProbeKind::Directory,
	},
	Probe {
		id: "cargo",
		name: "Cargo.toml",
		kind: ProbeKind::File,
	},
];
#[derive(Debug)]
pub enum Task {
	Index(PathBuf),
	GenerateConfig(PathBuf),
	Scan(PathBuf),
}
pub async fn worker(mut rx: mpsc::Receiver<Task>) {
	while let Some(task) = rx.recv().await {
		match task {
			Task::Index(path) => {
				// Create an index of something....
				println!("Indexing {:?}", path);
				// simulate work
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				println!("Finished {:?}", path);
			}
			Task::GenerateConfig(path) => {
				println!("Generating config {:?}", path);
			}
			Task::Scan(path) => {
				println!("Scanning {:?}", path);
			}
		}
	}
}
#[derive(Debug)]
pub struct FsWalker {
	root: PathBuf,
	target: PathBuf,
}
impl FsWalker {
	pub fn new(target: impl Into<PathBuf>) -> Self {
		let target = target.into();
		Self {
			root: Self::filesystem_root(&target),
			target,
		}
	}
	pub fn walk_up_to_target<F>(&self, mut visit: F) -> std::io::Result<()>
	where
		F: FnMut(&Path),
	{
		let mut current = self.root.clone();
		loop {
			visit(&current);
			if current == self.target {
				break;
			}
			let next = current.join(
				self
					.target
					.strip_prefix(&current)
					.unwrap()
					.components()
					.next()
					.unwrap(),
			);
			current = next;
		}
		Ok(())
	}
	fn filesystem_root(path: &Path) -> PathBuf {
		path.ancestors().last().unwrap().to_path_buf()
	}
}
pub enum Location {
	File {
		path: PathBuf,
		inode: Option<u64>,
	},
	Git {
		repo: String,
		commit: String,
		path: String,
	},
	Remote {
		uri: String,
	},
}
