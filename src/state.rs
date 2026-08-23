pub use crate::prelude::*;
use cli::prelude::{CliCommand, Context as CliContext, FormatArgs};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct EstateState {
	pub starts: u64,
	pub longest_run: u64,
	pub status_checks: u64,
	pub started_at: u64,
	pub events_processed: u64,
	pub tasks_completed: u64,
	pub tasks_created: u64,
	pub files_indexed: u64,
}
impl Default for EstateState {
	fn default() -> Self {
		Self {
			starts: 0,
			longest_run: 0,
			status_checks: 0,
			started_at: 0,
			events_processed: 0,
			tasks_completed: 0,
			tasks_created: 0,
			files_indexed: 0,
		}
	}
}
impl EstateState {
	pub fn save_workspace(path: &PathBuf) {
		println!("💾 save_workspace not implemented yet: {:?}", path);
	}
	pub fn now() -> u64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs()
	}
}
impl EstateState {
	pub fn loadFromPath(path: impl AsRef<Path>) -> Result<Self> {
		let contents = fs::read_to_string(path)?;
		Ok(serde_json::from_str(&contents)?)
	}
}

impl EstateState {
	fn path() -> std::io::Result<PathBuf> {
		Ok(engine_data_dir()?.join("state.json"))
	}
	pub fn load() -> Self {
		let path = Self::path().expect("could not resolve daemon state path");
		if !path.exists() {
			return Self::default();
		}
		tracing::info!("EstateState load path={:?}", path);
		let raw = fs::read_to_string(path).expect("failed reading daemon state");
		tracing::info!("EstateState raw={:?}", &raw);
		serde_json::from_str(&raw).expect("failed parsing daemon state")
	}
	pub fn save(state: &Self) {
		let path = Self::path().expect("could not resolve daemon state path");
		eprintln!("pathpath {:?}", path);
		let json = serde_json::to_string_pretty(state).expect("failed serializing daemon state");
		fs::write(path, json).expect("failed writing daemon state");
	}
	// pub fn record_status_check() {
	// 	let mut state = Self::load();
	// 	state.status_checks += 1;
	// 	Self::save(&state);
	// }
}
