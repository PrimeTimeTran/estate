//! Core domain types and workspace discovery.
//! # Description
//! This module contains:
//!
//! - [`Estate`]
//! - [`Node`]
//! - [`Resource`]
//! - [`Relation`]
//! - [`EstateDiscovery`]

use crate::prelude::*;

/// Represents an Estate and its complete project state.
///
/// An [`Estate`] is the root entity for a project. It owns the project's
/// identity, scope, nodes, resources, relations, and bindings.
///
/// Each Estate has a globally unique [`Uuid`] and may optionally have a
/// parent Estate, allowing Estates to be organized hierarchically.
///
/// # Resources
///
/// Resources represent files or other external assets associated with the
/// Estate. They can be created, looked up, mutably accessed, and removed
/// through the resource methods on this type.
///
/// # Examples
///
/// ```
/// let estate = Estate::new("my-project".into(), Scope::default());
/// assert_eq!(estate.resources.len(), 0);
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Estate {
	pub bindings: Vec<Binding>,
	pub id: Uuid,
	pub name: String,
	pub nodes: Vec<Node>,
	pub parent: Option<Uuid>,
	pub relations: Vec<Relation>,
	pub resources: Vec<Resource>,
	pub scope: Scope,
}

/// Estate Constructors
///
/// Constructors of Estate Entities
impl Estate {
	/// Initializes an Estate Entity.
	///
	/// The Estate starts with no parent, nodes, resources, relations, or
	/// bindings.
	pub fn new(name: String, scope: Scope) -> Self {
		Self {
			bindings: Vec::new(),
			id: Uuid::now_v7(),
			name,
			nodes: Vec::new(),
			parent: None,
			relations: Vec::new(),
			resources: Vec::new(),
			scope,
		}
	}
}

impl Estate {
	/// Adds a resource to the Estate and returns its identifier.
	///
	/// The resource's existing [`Resource::id`] is preserved.
	pub fn create_resource(&mut self, resource: Resource) -> Uuid {
		let id = resource.id;
		self.resources.push(resource);
		id
	}

	/// Returns a reference to the resource with the given identifier.
	///
	/// Returns `None` if the Estate does not contain a matching resource.
	pub fn resource(&self, id: Uuid) -> Option<&Resource> {
		self.resources.iter().find(|r| r.id == id)
	}

	/// Returns a mutable reference to the resource with the given identifier.
	///
	/// Returns `None` if the Estate does not contain a matching resource.
	pub fn resource_mut(&mut self, id: Uuid) -> Option<&mut Resource> {
		self.resources.iter_mut().find(|r| r.id == id)
	}

	/// Removes the resource with the given identifier from the Estate.
	///
	/// Returns the removed resource if it existed, otherwise `None`.
	pub fn remove_resource(&mut self, id: Uuid) -> Option<Resource> {
		let index = self.resources.iter().position(|r| r.id == id)?;
		Some(self.resources.remove(index))
	}
}

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
/// Data Store used while initializing an Estate Context
#[derive(Clone, Debug)]
pub struct EstateDiscovery {
	pub store: DiscoveryStore,
	// pub tasks: DiscoveryStore,
	pub task_tx: mpsc::Sender<DiscoveryTask>,
}
impl Default for EstateDiscovery {
	fn default() -> Self {
		Self {
			task_tx: Self::prepare(),
			store: DiscoveryStore::default(),
		}
	}
}
impl EstateDiscovery {
	// pub fn init(probes: ProbeSet) -> std::io::Result<DiscoveryStore> {
	// [*] Hard coded until discovery is stabilizes
	pub fn init() -> std::io::Result<DiscoveryStore> {
		let cwd = std::env::current_dir()?;
		let mut discovery = Self::default();
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
	pub fn prepare() -> mpsc::Sender<DiscoveryTask> {
		let (tx, rx) = mpsc::channel::<DiscoveryTask>(100);
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
/// Entity representing the MCP tools available
pub enum Tool {}
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct DiscoveryStore {
	pub items: Vec<DiscoveryItem>,
}
#[derive(Debug)]
pub enum DiscoveryEvent {
	Found(RawDiscovery),
	StartTask(DiscoveryTask),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EditorKind {
	Zed,
	// NeoVim,
	// Lapce,
	// VSCode,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
#[derive(Debug)]
/// Async tasks triggerable by events
pub enum DiscoveryTask {
	Index(PathBuf),
	GenerateConfig(PathBuf),
	Scan(PathBuf),
}
pub async fn worker(mut rx: mpsc::Receiver<DiscoveryTask>) {
	while let Some(task) = rx.recv().await {
		match task {
			DiscoveryTask::Index(path) => {
				// Create an index of something....
				println!("Indexing {:?}", path);
				// simulate work
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				println!("Finished {:?}", path);
			}
			DiscoveryTask::GenerateConfig(path) => {
				println!("Generating config {:?}", path);
			}
			DiscoveryTask::Scan(path) => {
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
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
/// Result produced by the discovery process.
///
/// Contains the discovered items and metadata about the scan.
pub struct DiscoveryResult {
	workspace: Workspace,
	packages: Vec<Package>,
	files: Vec<PathBuf>,
	ignored: Vec<PathBuf>,
}
// struct Workspace;
struct Package;
