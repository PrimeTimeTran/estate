use crate::{app::*, native::agent::FileInfo};

#[derive(Debug, Default, Clone)]
pub struct WorkspaceContext {
	pub files: Vec<FileInfo>,
}

impl WorkspaceContext {
	pub fn new(files: Vec<FileInfo>) -> Self {
		Self { files }
	}
}

impl WorkspaceContext {
	pub fn load() -> Result<Self> {
		Ok(Self { files: vec![] })
	}
}
