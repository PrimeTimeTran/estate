use crate::{
	agent::SystemEvent,
	job::{AgentTask, TaskResult},
};

#[derive(Clone, Debug)]
pub enum RuntimeEvent {
	System(SystemEvent),
	Agent(AgentEvent),
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
	NewTask { task: AgentTask },
	Thinking { task: AgentTask },
	Working { task: AgentTask, message: String },
	Finished { result: TaskResult },
	TaskEvent { task: AgentTask, event: TaskEvent },
}

#[derive(Debug, Clone)]
pub enum TaskEvent {
	Thinking,
	Started,
	Log(String),
	Working(String),
	Finished(String),
	Error(String),
}
