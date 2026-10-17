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
use serde::{Deserialize, Serialize};

use anyhow::Result;
use notify::{Event, EventKind};
use std::path::{Path, PathBuf};

mod r#const;
use r#const::*;

mod r#enum;
pub use r#enum::*;

mod r#fn;
pub use r#fn::*;

pub mod fmt;
pub use fmt::*;

pub mod agent;
pub use agent::*;

pub mod agent_runtime;
pub use agent_runtime::*;

pub mod log;
pub use log::*;

pub mod prompt;
pub use prompt::*;

pub mod system;
pub use system::*;

pub mod tool;
pub use tool::*;
