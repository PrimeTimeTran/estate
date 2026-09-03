use std::time::SystemTime;

use crate::prelude::*;

#[cfg(feature = "native")]
use crate::native::job::*;

#[derive(Debug, Clone, Eq, Deserialize, PartialEq, Serialize)]
pub struct Task {
	pub id: TaskId,
	pub name: String,
	pub kind: TaskKind,
	pub status: TaskStatus,
}

#[derive(Debug)]
pub struct TaskManager {
	#[cfg(feature = "native")]
	runtime: TaskManagerRuntime,

	state: TaskManagerState,
}

#[derive(Debug, Default)]
pub struct TaskManagerState {
	pub dirty: bool,
	pub error: Option<String>,
	pub last_loaded: Option<SystemTime>,
	pub state: Option<EstateState>,
	pub tasks: HashMap<TaskId, Task>,

	#[cfg(feature = "native")]
	pub state_path: PathBuf,
}
