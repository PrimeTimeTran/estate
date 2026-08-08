use std::path::PathBuf;

pub struct WorkspaceContext {
	pub root: PathBuf,
	pub estate: Option<PathBuf>,
}

pub struct RuntimeContext {
	pub engine_dir: PathBuf,
	pub connected: bool,
}

#[derive(Debug)]
pub struct Context {
	pub source: ContextSource,

	// Where the user is operating
	pub workspace: PathBuf,

	// Global user estate (~/.estate)
	pub estate_root: PathBuf,

	// Engine internals (cache, daemon state, registry)
	pub engine_root: PathBuf,
}

#[derive(Debug)]
pub enum ContextSource {
	Cli,
	ZedEditor,
	CompilerPipeline,
	KnowledgeBase,
}

impl Context {
	pub fn new(source: ContextSource) -> std::io::Result<Self> {
		Ok(Self {
			source,
			workspace: std::env::current_dir()?,
			estate_root: crate::daemon::resolver::global_estate_dir()?,
			engine_root: crate::daemon::resolver::engine_data_dir()?,
		})
	}
}
