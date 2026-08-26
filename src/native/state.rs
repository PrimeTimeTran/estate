pub use crate::prelude::*;

// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// #[serde(default)]
// pub struct EstateState {
// 	pub revision: u64,

// 	pub starts: u64,
// 	pub longest_run: u64,
// 	pub status_checks: u64,
// 	pub started_at: u64,
// 	pub events_processed: u64,
// 	pub tasks_completed: u64,
// 	pub tasks_created: u64,
// 	pub files_indexed: u64,
// 	pub jobs: VecDeque<Job>,
// }
// impl Default for EstateState {
// 	fn default() -> Self {
// 		Self {
// 			revision: 0,
// 			jobs: VecDeque::new(),
// 			starts: 0,
// 			longest_run: 0,
// 			status_checks: 0,
// 			started_at: 0,
// 			events_processed: 0,
// 			tasks_completed: 0,
// 			tasks_created: 0,
// 			files_indexed: 0,
// 		}
// 	}
// }
// impl EstateState {
// 	pub fn save_workspace(path: &PathBuf) {
// 		println!("💾 save_workspace not implemented yet: {:?}", path);
// 	}
// 	pub fn now() -> u64 {
// 		std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
// 	}
// }
// impl EstateState {
// 	pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
// 		let contents = fs::read_to_string(path)?;
// 		Ok(serde_json::from_str(&contents)?)
// 	}
// }

// impl EstateState {
// 	fn path() -> std::io::Result<PathBuf> {
// 		Ok(engine_data_dir()?.join("state.json"))
// 	}
// 	pub fn load() -> Self {
// 		let path = Self::path().expect("could not resolve daemon state path");

// 		if !path.exists() {
// 			tracing::warn!("EstateState does not exist: {:?}", path);
// 			return Self::default();
// 		}

// 		let raw = fs::read_to_string(&path).expect("failed reading daemon state");

// 		match serde_json::from_str(&raw) {
// 			Ok(state) => state,
// 			Err(error) => {
// 				tracing::error!(
//                 %error,
//                 path = ?path,
//                 "failed to parse EstateState JSON"
//             );

// 				panic!("EstateState is corrupted");
// 			}
// 		}
// 	}
// 	pub fn save(state: &Self) {
// 		let path = Self::path().expect("could not resolve daemon state path");
// 		tracing::info!("💾 EstateState received: {:?}", path);
// 		let json = serde_json::to_string_pretty(state).expect("failed serializing daemon state");
// 		fs::write(path, json).expect("failed writing daemon state");
// 	}
// 	// pub fn record_status_check() {
// 	// 	let mut state = Self::load();
// 	// 	state.status_checks += 1;
// 	// 	Self::save(&state);
// 	// }
// }
