use crate::{
	app::*,
	prelude::{Uuid, *},
};

#[derive(Debug, Clone, Eq, Deserialize, PartialEq, Serialize)]
pub struct Job {
	pub id: Uuid,
	pub task_id: Uuid,
	pub kind: TaskKind,
	pub status: JobStatus,

	pub created_at: u64,
	pub started_at: Option<u64>,
	pub completed_at: Option<u64>,
}
impl JobStatus {
	pub fn label(self) -> &'static str {
		match self {
			Self::Cancelled => "Cancelled",
			Self::Completed => "Completed",
			Self::Failed => "Failed",
			Self::Pending => "Pending",
			Self::Running => "Running",
			_ => todo!("icon"),
		}
	}
	pub fn icon(self) -> &'static str {
		match self {
			Self::Cancelled => "⊘",
			Self::Completed => "✓",
			Self::Failed => "✗",
			Self::Pending => "○",
			Self::Running => "●",
			_ => todo!("icon"),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum JobStatus {
	Pending,
	Running,
	Completed,
	Failed,
	Cancelled,
	Interrupted,
}
