// | Concept                         | Name           | Meaning                       |
// | ------------------------------- | -------------- | ----------------------------- |
// | What needs doing                | `Task`         | Logical unit of work          |
// | An execution of it              | `Job`          | Concrete background execution |
// | Oversees them                   | `TaskManager`  | Coordinates tasks/jobs        |
// | Individual background execution | `Job`          | Has lifecycle/state           |
// | UI representation               | `Task` / `Job` | Shows pending/running/etc.    |

use crate::{
	model::task::TaskResult,
	prelude::{anyhow::anyhow, *},
	sdlc::AiSession,
};
use notify::{Event, EventKind};

mod r#const;
use r#const::*;

mod r#enum;
pub use r#enum::*;

mod r#fn;
pub use r#fn::*;

mod fmt;
use fmt::*;

pub mod agent;
pub mod agent_runtime;
pub mod prompt;
pub mod system;
pub mod tool;

pub use agent::*;
pub use agent_runtime::*;

pub use prompt::*;
pub use system::*;
pub use tool::*;

pub mod log;
pub use log::*;
