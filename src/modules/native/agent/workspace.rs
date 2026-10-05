use crate::native::agent::FileInfo;
use crate::prelude::*;

pub fn format_workspace(workspace: &WorkspaceContext) -> String {
	let mut output = String::new();
	output.push_str(&format!("CWD: {}\n", workspace.cwd.display()));
	if workspace.files.is_empty() {
		output.push_str("FILES: none discovered\n");
	} else {
		output.push_str("FILES:\n");
		for file in &workspace.files {
			output.push_str(&format!("- {}\n", file.path));
		}
	}

	output
}
fn format_history(history: &[AgentObservation]) -> String {
	if history.is_empty() {
		return "No actions have been performed yet.".into();
	}

	let mut output = String::new();

	for (index, observation) in history.iter().enumerate() {
		output.push_str(&format!("{}. {:?}\n", index + 1, observation));
	}

	output
}
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