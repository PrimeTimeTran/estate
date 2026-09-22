pub mod agent;
#[path = "./agent-event.rs"]
pub mod agent_event;
pub mod prompt;

#[path = "./runtime.rs"]
pub mod agent_runtime;
pub mod system;
pub mod tool;
pub mod workspace;

pub use agent::*;
pub use agent_runtime::*;
pub use prompt::*;
pub use system::*;
pub use tool::*;
pub use workspace::*;

// | Concept                         | Name           | Meaning                       |
// | ------------------------------- | -------------- | ----------------------------- |
// | What needs doing                | `Task`         | Logical unit of work          |
// | An execution of it              | `Job`          | Concrete background execution |
// | Oversees them                   | `TaskManager`  | Coordinates tasks/jobs        |
// | Individual background execution | `Job`          | Has lifecycle/state           |
// | UI representation               | `Task` / `Job` | Shows pending/running/etc.    |

use crate::prelude::*;

use notify::{Event, EventKind};

#[derive(Debug, Clone)]
pub enum Artifact {
	FileRead { path: String, content: String },
	FileWrite { path: String },
	Observation(String),
	ToolOutput(String),
}

#[derive(Debug, Clone)]
pub struct AgentTask {
	pub id: Uuid,
	pub prompt: String,
}

impl AgentTask {
	pub fn new(prompt: String) -> Self {
		Self {
			id: Uuid::new_v4(),
			prompt,
		}
	}
}
