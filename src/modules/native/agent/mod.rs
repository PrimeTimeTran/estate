// | Concept                         | Name           | Meaning                       |
// | ------------------------------- | -------------- | ----------------------------- |
// | What needs doing                | `Task`         | Logical unit of work          |
// | An execution of it              | `Job`          | Concrete background execution |
// | Oversees them                   | `TaskManager`  | Coordinates tasks/jobs        |
// | Individual background execution | `Job`          | Has lifecycle/state           |
// | UI representation               | `Task` / `Job` | Shows pending/running/etc.    |

use notify::{Event, EventKind};

use crate::{
	model::task::TaskResult,
	prelude::{anyhow::anyhow, *},
	sdlc::SdlcSession,
};

mod r#const;
use r#const::*;

mod fmt;
use fmt::*;

mod r#fn;
use r#fn::*;

pub mod agent;
#[path = "./agent-event.rs"]
pub mod agent_event;
pub use agent_event::{AgentEvent, RuntimeEvent};

pub mod prompt;

#[path = "./runtime.rs"]
pub mod agent_runtime;
pub mod system;
pub mod tool;
pub mod workspace;


use crate::prelude::*;
pub use agent::*;
pub use agent_runtime::*;

pub use prompt::*;
pub use system::*;
pub use tool::*;
pub use workspace::*;

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
	pub fn from_session(session: &SdlcSession) -> Result<Self> {
		Ok(Self {
			id: Uuid::new_v4(),
			prompt: session.goal.clone(),
		})
	}
}
impl AgentTask {
	pub fn new(prompt: String) -> Self {
		Self {
			id: Uuid::new_v4(),
			prompt,
		}
	}
}
