use crate::{
	model::resolver::{crate_root, ws_path},
	native::*,
	prelude::*,
};

use anyhow::{Context, Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Copy)]
pub enum SrcArtifact {
	Intent,
	Spec,
	Plan,
	Test,
	Build,
	QA,
	Progress,
}

#[derive(Debug, Clone, Copy)]
pub enum SpecialFile {
	AiTemplateDir,
	EstateManifest,
	HostContext,
	WriteCurrent,
	WriteDir,
	SessionsIndex,
}
impl FS {
	pub fn find(scope: FW, resource: Resource) -> Result<Option<PathBuf>> {
		todo!("FS find FW resource: Resource")
	}
	pub fn load<T>(path: impl AsRef<Path>) -> Result<Option<T>>
	where
		T: serde::de::DeserializeOwned,
	{
		let path = path.as_ref();

		Self::ensure_parent(path.to_path_buf())?;

		if !path.exists() {
			return Ok(None);
		}

		let contents = fs::read_to_string(path)?;

		match path.extension().and_then(|ext| ext.to_str()) {
			Some("json") => Ok(Some(serde_json::from_str(&contents)?)),
			Some("toml") => Ok(Some(toml::from_str(&contents)?)),
			Some(ext) => Err(anyhow!(
				"unsupported file format '.{ext}' for {}",
				path.display()
			)),
			None => Err(anyhow!(
				"cannot determine file format for {}",
				path.display()
			)),
		}
	}
	pub fn read(path: impl AsRef<Path>) -> Result<String> {
		let path = path.as_ref();

		Self::ensure_parent(path.to_path_buf())
			.with_context(|| format!("ensure parent for {}", path.display()))?;

		fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
	}
	pub fn write(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = Self::ensure_parent(path)?;

		fs::write(&path, contents).with_context(|| format!("write {}", path.display()))?;

		Ok(())
	}
	pub fn append(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = Self::ensure_parent(path)?;

		let mut file = fs::OpenOptions::new()
			.create(true)
			.append(true)
			.open(&path)
			.with_context(|| format!("open {}", path.display()))?;

		file
			.write_all(contents.as_ref())
			.with_context(|| format!("append {}", path.display()))?;

		Ok(())
	}
	pub fn save<T>(path: impl Into<PathBuf>, value: &T) -> Result<()>
	where
		T: Serialize,
	{
		let path = Self::ensure_parent(path)?;
		let contents = serde_json::to_string_pretty(value)?;
		fs::write(&path, contents).with_context(|| format!("write {}", path.display()))?;
		Ok(())
	}

	pub fn delete(path: impl AsRef<Path>) -> Result<()> {
		let path = path.as_ref();
		Self::ensure_parent(path.to_path_buf())?;
		if path.exists() {
			fs::remove_file(path)?;
		}
		Ok(())
	}
	pub fn ensure_parent(path: impl Into<PathBuf>) -> Result<PathBuf> {
		let path = path.into();
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}
		Ok(path)
	}
	pub fn ensure_dir(path: impl AsRef<Path>) -> Result<PathBuf> {
		let path = path.as_ref();
		eprintln!("ensure_dir:");
		eprintln!("  path = {path:?}");
		eprintln!("  cwd  = {:?}", std::env::current_dir()?);
		fs::create_dir_all(path)?;
		Ok(path.to_path_buf())
	}
	pub fn exists(path: impl AsRef<Path>) -> bool {
		path.as_ref().exists()
	}
	pub fn create(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = path.into();
		if path.exists() {
			return Err(anyhow!("file already exists: {}", path.display()));
		}
		Self::ensure_parent(&path)?;
		fs::write(path, contents)?;
		Ok(())
	}
	pub fn update(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = path.into();
		Self::ensure_parent(&path)?;
		fs::write(path, contents)?;
		Ok(())
	}
}

impl SrcArtifact {
	pub fn name(self) -> &'static str {
		match self {
			Self::Intent => "intent.md",
			Self::Spec => "spec.md",
			Self::Plan => "plan.md",
			Self::Test => "tests.md",
			Self::Build => "build.md",
			Self::QA => "verification.md",
			Self::Progress => "progress.md",
		}
	}
	pub fn path(self, session: impl AsRef<Path>) -> PathBuf {
		session.as_ref().join(self.name())
	}
	pub fn read(self, session: impl AsRef<Path>) -> Result<String> {
		FS::read(self.path(session))
	}
	pub fn write(self, session: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
		FS::write(self.path(session), contents)
	}
	pub fn append(self, session: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
		FS::append(self.path(session), contents)
	}
}
impl SpecialFile {
	pub fn load<T>(self) -> Result<Option<T>>
	where
		T: serde::de::DeserializeOwned,
	{
		FS::load(self.path()?)
	}
	pub fn path(self) -> Result<PathBuf> {
		let root = ws_path()?;
		Ok(match self {
			Self::AiTemplateDir => root.join("ai/template"),
			Self::WriteCurrent => root.join("crates/estate/log/sdlc.current.json"),
			Self::WriteDir => root.join("crates/estate/log"),
			Self::SessionsIndex => root.join("crates/estate/log/sdlc.session.index.json"),
			Self::EstateManifest => root.join("estate.toml"),
			Self::HostContext => crate_root().join("host.env.context.json"),
		})
	}
}

pub struct FS;

#[derive(Debug, Clone, Copy)]
enum Platform {
	MacOS,
	Windows,
	Linux,
}
#[derive(Debug, Clone, Copy)]
pub enum Appp {
	Estate,
	VSCode,
	Zed,
	RustRover,
}
impl Appp {
	pub fn fw(self, fw: FW) -> Result<PathBuf> {
		match self {
			Self::Estate => EstateFW::path(fw),
			_ => todo!(),
		}
	}
}
#[derive(Debug, Clone, Copy)]
pub enum FW {
	Log,
	Tmp,

	Session,
	AiTemplates,

	Settings,
	Keybindings,
	Extensions,
	Workspace,
	Index,
}
#[derive(Debug, Clone, Copy)]
pub struct Kontex {
	platform: Platform,
	app: Appp,
}
impl Kontex {
	pub fn new(app: Appp) -> Result<Self> {
		let platform = if cfg!(target_os = "macos") {
			Platform::MacOS
		} else if cfg!(target_os = "windows") {
			Platform::Windows
		} else if cfg!(target_os = "linux") {
			Platform::Linux
		} else {
			return Err(anyhow!("unsupported platform"));
		};
		Ok(Self { platform, app })
	}
	pub fn session_read(&self, name: &str) -> Result<String> {
		let path = self.path(FW::Session)?.join(name);
		FS::read(path)
	}
	pub fn session_write(&self, title: &str, contents: impl AsRef<[u8]>) -> Result<PathBuf> {
		let sessions = FS::ensure_dir(self.path(FW::Session)?)?;
		let date = Local::now().format("%Y-%m-%d");
		let path = sessions.join(format!("{date}.{title}"));
		FS::write(&path, contents)?;
		Ok(path)
	}
	pub fn session_load<T>(&self) -> Result<Option<T>>
	where
		T: serde::de::DeserializeOwned,
	{
		let path = self.path(FW::Log)?.join("sdlc.current.json");
		FS::load(path)
	}
	pub fn session_save<T>(&self, value: &T) -> Result<()>
	where
		T: serde::Serialize,
	{
		let path = self.path(FW::Log)?.join("sdlc.current.json");
		FS::save(path, value)
	}
	pub fn log_read(&self, name: &str) -> Result<String> {
		let path = self.path(FW::Log)?.join(name);
		FS::read(path)
	}
	pub fn log_write(&self, name: &str, contents: impl AsRef<[u8]>) -> Result<PathBuf> {
		let log = FS::ensure_dir(self.path(FW::Log)?)?;
		let path = log.join(name);
		FS::write(&path, contents)?;
		Ok(path)
	}
	pub fn path(self, fw: FW) -> Result<PathBuf> {
		self.app.fw(fw)
	}
	pub fn keybindings(self, root: PathBuf) -> Result<PathBuf> {
		match (self.platform, self.app) {
			(Platform::MacOS, Appp::Zed) => Ok(root.join("apps/zed/keybindings")),
			(Platform::Windows, Appp::Zed) => Ok(root.join("apps/zed/keybindings")),
			(Platform::MacOS, Appp::VSCode) => Ok(root.join("apps/vscode/keybindings")),
			(Platform::Windows, Appp::VSCode) => Ok(root.join("apps/vscode/keybindings")),
			_ => Err(anyhow!(
				"keybindings not supported for {:?} on {:?}",
				self.app,
				self.platform
			)),
		}
	}
}
pub struct EstateFW;

impl EstateFW {
	fn path(fw: FW) -> Result<PathBuf> {
		let root = PathBuf::from("/Users/future/kb/project/crates/estate/log");
		Ok(match fw {
			FW::Log => root.clone(),
			FW::Tmp => root.clone(),
			FW::Session => root.clone(),
			FW::AiTemplates => root.clone(),
			FW::Settings => root.clone(),
			FW::Index => root.clone(),
			_ => return Err(anyhow!("{fw:?} is not an Estate filesystem resource")),
		})
	}
}
