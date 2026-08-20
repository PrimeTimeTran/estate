use crate::prelude::*;
use revelation::analyzer::Workspace;

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Estate {
	pub id: Uuid,
	pub name: String,
	pub parent: Option<Uuid>,
	pub scope: Scope,
	pub nodes: Vec<Node>,
	// pub nodes: Vec<Uuid>,
	pub resources: Vec<Resource>,
	pub relations: Vec<Relation>,
	pub bindings: Vec<Binding>,
}
impl Estate {
	pub fn new(name: String, scope: Scope) -> Self {
		Self {
			id: Uuid::now_v7(),
			name,
			parent: None,
			scope,
			nodes: Vec::new(),
			resources: Vec::new(),
			relations: Vec::new(),
			bindings: Vec::new(),
		}
	}
}
impl Estate {
	pub fn create_resource(&mut self, resource: Resource) -> Uuid {
		let id = resource.id;
		self.resources.push(resource);
		id
	}

	pub fn resource(&self, id: Uuid) -> Option<&Resource> {
		self.resources.iter().find(|r| r.id == id)
	}

	pub fn resource_mut(&mut self, id: Uuid) -> Option<&mut Resource> {
		self.resources.iter_mut().find(|r| r.id == id)
	}

	pub fn remove_resource(&mut self, id: Uuid) -> Option<Resource> {
		let index = self.resources.iter().position(|r| r.id == id)?;
		Some(self.resources.remove(index))
	}
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
#[derive(Clone, Debug)]
pub struct EstateDiscovery {
	pub store: DiscoveryStore,
	// pub tasks: DiscoveryStore,
	pub task_tx: mpsc::Sender<Task>,
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct DiscoveryStore {
	pub items: Vec<DiscoveryItem>,
}
#[derive(Debug)]
pub enum DiscoveryEvent {
	Found(RawDiscovery),
	StartTask(Task),
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
pub struct DiscoveryResult {
	workspace: Workspace,
	packages: Vec<Package>,
	files: Vec<PathBuf>,
	ignored: Vec<PathBuf>,
}
// struct Workspace;
struct Package;
