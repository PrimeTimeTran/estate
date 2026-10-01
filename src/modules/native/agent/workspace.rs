use crate::native::agent::FileInfo;
use crate::prelude::*;
#[derive(Debug, Default, Clone)]
pub struct WorkspaceContext {
	pub files: Vec<FileInfo>,
}

impl WorkspaceContext {
	pub fn new(files: Vec<FileInfo>) -> Self {
		Self { files }
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

		Ok(Self { files })
	}

	pub fn load() -> anyhow::Result<Self> {
		Ok(Self { files: vec![] })
	}
}
