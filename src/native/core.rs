use crate::{
	app::{Runtime, model, *},
	data,
};

impl<R: Runtime> model::EstateEngine<R> {
	pub async fn format(self, args: &FormatArgs) -> Result<String, Error> {
		LintDaemon.run(&args).await;
		Ok("Success".to_string())
	}
}
#[derive(Clone, Debug)]
pub struct EstateDiscovery {
	pub store: DiscoveryStore,
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
	fn discover_files(&mut self, root: &Path) -> std::io::Result<()> {
		self.walk_files(root)
	}
	fn walk_files(&mut self, dir: &Path) -> std::io::Result<()> {
		for entry in std::fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_dir() {
				self.walk_files(&path)?;
				continue;
			}
			if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
				self.store.files.push(path);
			}
		}
		Ok(())
	}
}
impl EstateDiscovery {
	pub fn init() -> std::io::Result<DiscoveryStore> {
		let cwd = std::env::current_dir()?;
		let mut discovery = Self::default();
		discovery.discover_personal(&cwd)?;
		discovery.discover_files(&cwd)?;
		discovery.discover_config(&cwd)?;
		Ok(discovery.store)
	}
	fn discover_config(&mut self, cwd: &Path) -> std::io::Result<()> {
		if let Some(home) = dirs::home_dir() {
			let config_dir = home.join(HOME_DIR);
			for name in INTRINSIC_FILES {
				let path = config_dir.join(name);
				if path.is_file() {
					self.store.items.push(DiscoveryItem::Config(path));
				}
			}
		}
		walk_root_to_path(cwd, |dir| {
			let path = dir.join(WORKSPACE_SETTINGS);
			if path.is_file() {
				// tracing::info!(?path, "discovered workspace config");
				self.store.items.push(DiscoveryItem::Config(path));
			}
			WalkControl::Continue
		})?;
		Ok(())
	}
	fn discover_personal(&mut self, cwd: &Path) -> std::io::Result<()> {
		walk_root_to_path(cwd, |dir| {
			for probe in PROBES_PERSONAL {
				if let Some(path) = Self::probe(dir, probe) {
					let raw = RawDiscovery { probe, path };
					self.handle(DiscoveryEvent::Found(raw));
				}
			}
			WalkControl::Continue
		})?;
		Ok(())
	}
	fn handle(&mut self, event: DiscoveryEvent) {
		match event {
			DiscoveryEvent::StartTask(_) => {}
			DiscoveryEvent::Found(raw) => match raw.probe.id {
				"cargo" => {
					self.store.add_directory(FrameworkKind::Cargo, raw.path);
				}
				"npm" => {
					self.store.add_directory(FrameworkKind::Npm, raw.path);
				}
				"estate" => {
					self.store.add_directory(FrameworkKind::Estate, raw.path);
				}
				_ => {}
			},
		}
	}
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
			"zed" => Some(DiscoveryItem::Editor(EditorKind::Zed)),
			"cargo" => Some(DiscoveryItem::CargoProject(raw.path)),
			"npm" => Some(DiscoveryItem::Project(raw.path)),
			"Cargo.toml" => Some(DiscoveryItem::CargoProject(raw.path)),
			"package.json" => Some(DiscoveryItem::Project(raw.path)),
			"zed" => Some(DiscoveryItem::PackageManager(raw.path)),
			"estate-settings" => Some(DiscoveryItem::Settings(raw.path)),
			"estate-keymap" => Some(DiscoveryItem::KeyMap(raw.path)),
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
			DiscoveryEvent::StartTask(raw) => {}
			DiscoveryEvent::Found(raw) => match raw.probe.id {
				"cargo" => {
					self.store.add_directory(FrameworkKind::Cargo, raw.path);
				}
				"npm" => {
					self.store.add_directory(FrameworkKind::Npm, raw.path);
				}

				"git" => {
					self.store.add_directory(FrameworkKind::Git, raw.path);
				}
				_ => {
					if let Some(item) = Self::resolve(raw) {
						self.store.items.push(item);
					}
				}
			},
		}
	}
}
#[derive(Debug, Serialize, Deserialize)]
pub enum Tool {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirectoryDiscovery {
	pub estate: Vec<PathBuf>,
	pub git: Vec<PathBuf>,
	pub cargo: Vec<PathBuf>,
	pub npm: Vec<PathBuf>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FrameworkKind {
	Estate,
	Git,
	Cargo,
	Npm,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscoveryStore {
	pub items: Vec<DiscoveryItem>,
	pub files: Vec<PathBuf>,
	pub directories: HashMap<FrameworkKind, Vec<PathBuf>>,
	pub configs: Vec<PathBuf>,
}
impl DiscoveryStore {
	pub fn add_directory(&mut self, kind: FrameworkKind, path: PathBuf) {
		self.directories.entry(kind).or_default().push(path);
	}
	pub fn add_file(&mut self, path: PathBuf) {
		self.files.push(path);
	}
	pub fn types(&self) -> Vec<String> {
		let mut types = std::collections::HashSet::new();
		for path in &self.files {
			if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
				types.insert(format!(".{ext}"));
			}
		}
		let mut types: Vec<String> = types.into_iter().collect();
		types.sort();
		types
	}
	pub fn filtered_resources(&self) -> Vec<String> {
		self
			.items
			.iter()
			.filter_map(|item| match item {
				DiscoveryItem::Config(path) => Some(path.to_string_lossy().into_owned()),
				_ => None,
			})
			.collect()
	}
	fn get_framework_paths(&self, kind: FrameworkKind) -> &[PathBuf] {
		self
			.directories
			.get(&kind)
			.map(Vec::as_slice)
			.unwrap_or(&[])
	}
	pub fn write_discovery_result(&mut self) -> Result<()> {
		let path = dirs::home_dir()
			.ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?
			.join(INDEX_PATH);
		tracing::debug!(?path, "writing discovery result");
		let mut master: serde_json::Value = if path.exists() {
			tracing::debug!("loading existing master.json");
			let contents = std::fs::read_to_string(&path)
				.map_err(|error| anyhow::anyhow!("failed to read {:?}: {error}", path))?;
			if contents.trim().is_empty() {
				tracing::debug!("master.json is empty, creating default");
				data::master()
			} else {
				serde_json::from_str::<serde_json::Value>(&contents)
					.map_err(|error| anyhow::anyhow!("invalid master.json: {error}"))?
			}
		} else {
			tracing::debug!("master.json does not exist, creating default");
			data::master()
		};
		if !master.is_object() {
			master = data::master();
		}
		self.apply_discovery(&mut master)?;
		let contents = serde_json::to_string_pretty(&master)?;
		let parent = path
			.parent()
			.ok_or_else(|| anyhow::anyhow!("master.json has no parent"))?;
		std::fs::create_dir_all(parent)?;
		let tmp = path.with_extension("json.tmp");
		tracing::debug!(?tmp, "writing temporary master.json");
		std::fs::write(&tmp, &contents)
			.map_err(|error| anyhow::anyhow!("failed to write {:?}: {error}", tmp))?;
		std::fs::rename(&tmp, &path)
			.map_err(|error| anyhow::anyhow!("failed to replace {:?}: {error}", path))?;
		tracing::debug!(
			files = self.files.len(),
			items = self.items.len(),
			"discovery result written"
		);
		Ok(())
	}
	pub fn apply_discovery(&mut self, mas: &mut serde_json::Value) -> Result<()> {
		let unique_files: std::collections::HashSet<_> = self.files.iter().collect();
		mas["metrics"]["files"] = serde_json::json!(self.files.len());
		mas["metrics"]["files.unique"] = serde_json::json!(unique_files.len());
		mas["estate"]["files"] = serde_json::to_value(self.files.clone())?;
		let mut counts = std::collections::HashMap::<String, usize>::new();
		for file in self.files.clone() {
			if let Some(ext) = file.extension().and_then(|ext| ext.to_str()) {
				*counts.entry(format!(".{ext}")).or_default() += 1;
			}
		}
		let mut counter: Vec<serde_json::Value> = counts
			.iter()
			.map(|(ext, num)| {
				serde_json::json!({
					"ext": ext,
					"num": num,
				})
			})
			.collect();
		counter.sort_by(|a, b| a["ext"].as_str().cmp(&b["ext"].as_str()));
		let types = counter
			.iter()
			.filter_map(|item| item["ext"].as_str())
			.collect::<Vec<_>>()
			.join("|");
		mas["metrics"]["types"] = serde_json::json!(counts.len());
		mas["metrics"]["counter"] = serde_json::Value::Array(counter);
		mas["estate"]["types"] = serde_json::json!(types);

		let cargo_paths = self.get_framework_paths(FrameworkKind::Cargo);
		let estate_paths = self.get_framework_paths(FrameworkKind::Estate);
		let npm_paths = self.get_framework_paths(FrameworkKind::Npm);

		mas["metrics"]["projects"]["cargo"] = serde_json::json!(cargo_paths);
		mas["metrics"]["projects"]["npm"] = serde_json::json!(npm_paths);
		mas["metrics"]["projects"]["estate"] = serde_json::json!(estate_paths);
		let has_dotrepo = self
			.items
			.iter()
			.any(|item| matches!(item, DiscoveryItem::GitRepo(_)));
		mas["estate"]["onboarding"]["has.dotrepo"] = serde_json::json!(has_dotrepo);
		let config_sources: Vec<String> = self
			.items
			.iter()
			.filter_map(|item| match item {
				DiscoveryItem::Config(path) => Some(path.to_string_lossy().into_owned()),
				_ => None,
			})
			.collect();
		mas["config.active"]["sources"] = serde_json::json!(config_sources);
		Ok(())
	}
}
#[derive(Debug)]
pub enum DiscoveryEvent {
	Found(RawDiscovery),
	StartTask(DiscoveryTask),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorKind {
	Zed,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryItem {
	Estate(PathBuf),
	GitRepo(PathBuf),
	CargoProject(PathBuf),
	Editor(EditorKind),
	Project(PathBuf),
	PackageManager(PathBuf),
	Config(PathBuf),
	Settings(PathBuf),
	KeyMap(PathBuf),
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
///      Async tasks triggerable by events
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
				// println!("Indexing {:?}", path);
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
///      Result produced by the discovery process.
///
///      Contains the discovered items and metadata about the scan.
pub struct DiscoveryResult {
	pub workspace: Workspace,
	pub packages: Vec<Package>,
	pub files: Vec<PathBuf>,
	pub ignored: Vec<PathBuf>,
}
// struct Workspace;
struct Package;
