use crate::native::agent::FileInfo;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct WorkspaceContext {
	pub files: Vec<FileInfo>,
	pub cwd: PathBuf,
}
impl Default for WorkspaceContext {
	fn default() -> Self {
		Self {
			files: Vec::new(),
			cwd: std::env::current_dir().unwrap_or_default(),
		}
	}
}
impl fmt::Display for WorkspaceContext {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "CWD: {}", self.cwd.display())?;
		writeln!(f, "FILES: {}", self.files.len())?;
		if self.files.is_empty() {
			return Ok(());
		}
		for (i, file) in self.files.iter().take(10).enumerate() {
			writeln!(f, "  [{i}] {file:?}")?;
		}
		if self.files.len() > 10 {
			writeln!(f, "  ... {} more files", self.files.len() - 10)?;
		}
		Ok(())
	}
}
impl WorkspaceContext {
	pub fn new(files: Vec<FileInfo>) -> Self {
		Self {
			files,
			cwd: std::env::current_dir().unwrap_or_default(),
		}
	}
	pub fn from_session(session: &SdlcSession) -> Result<Self> {
		let mut files = Vec::new();
		for entry in std::fs::read_dir(&session.dir)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_file() {
				files.push(FileInfo::from_path(&path)?);
			}
		}
		Ok(Self {
			files,
			cwd: session.dir.clone(),
		})
	}
	pub fn from_sdlc_session(session: &SdlcSession) -> Result<Self> {
		tracing::info!("from_sdlc_sessionfrom_sdlc_session workspace");
		let cwd = session.dir.clone();
		let files = [
			SessionFile::Intent,
			SessionFile::Spec,
			SessionFile::Plan,
			SessionFile::Progress,
		]
		.into_iter()
		.map(|file| {
			let path = file.path(&cwd);
			FileInfo::from_path(&path)
		})
		.collect::<Result<Vec<_>>>()?;
		Ok(Self { cwd, files })
	}
	pub fn load() -> anyhow::Result<Self> {
		Ok(Self {
			files: Vec::new(),
			cwd: std::env::current_dir()?,
		})
	}
	pub fn from_cwd(cwd: impl Into<PathBuf>) -> Self {
		Self {
			cwd: cwd.into(),
			files: Vec::new(),
		}
	}
}
