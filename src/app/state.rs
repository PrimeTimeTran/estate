use crate::{app::job::Job, prelude::*};

pub trait StateStore: Send + Sync {
	fn load(&self) -> Result<EstateState>;
	fn save(&self, state: &EstateState) -> Result<()>;
}

// #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// pub struct Event {
// pub completed_at: u64,
// pub created_at: u64,
// pub id: String,
// pub kind: String,
// pub started_at: u64,
// pub status: String,
// pub task_id: String,
// }
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct EstateState {
	pub revision: u64,
	pub starts: u64,
	pub longest_run: u64,
	pub status_checks: u64,
	pub started_at: u64,
	pub events_processed: u64,
	pub tasks_completed: u64,
	pub tasks_created: u64,
	pub files_indexed: u64,
	pub session: Session,
	// pub jobs: Vec<Job>,
	pub jobs: VecDeque<Job>,
	// #[cfg(feature = "native")]
}

impl Default for EstateState {
	fn default() -> Self {
		Self {
			events_processed: 0,
			files_indexed: 0,
			longest_run: 0,
			jobs: VecDeque::new(),
			// #[cfg(feature = "native")]
			// jobs: vec![],
			revision: 0,
			started_at: 0,
			starts: 0,
			status_checks: 0,
			tasks_completed: 0,
			tasks_created: 0,
			session: Session::default(),
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

// pub struct WebStateStore;
// impl StateStore for WebStateStore {
// 	fn load(&self) -> Result<EstateState> {
// 		todo!("load")
// 	}
// 	fn save(&self, state: &EstateState) -> Result<()> {
// 		todo!("save")
// 	}
// }
