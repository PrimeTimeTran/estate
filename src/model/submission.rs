use crate::prelude::*;

pub struct CreateSubmission {
	pub user_id: String,
	pub problem_id: String,
	pub source: String,
	pub language: String,
}
pub struct UpdateSubmission {
	pub source: Option<String>,
	pub language: Option<String>,
}
pub struct SubmissionQuery {
	pub page: Option<i32>,
	pub page_size: Option<i32>,
	pub user_id: Option<String>,
	pub problem_id: Option<String>,
	pub status: Option<SubmissionStatus>,
	pub language: Option<String>,
}

pub struct SubmissionState {
	pub status: SubmissionStatus,
	pub id: Option<String>,
	pub result: Option<SubmissionResult>,
	pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SubmissionResult {
	Accepted,
	WrongAnswer,
	CompilationError,
	RuntimeError,
	TimeLimitExceeded,
	MemoryLimitExceeded,
	InternalError,
}

impl TryFrom<i32> for SubmissionResult {
	type Error = anyhow::Error;

	fn try_from(value: i32) -> Result<Self, Self::Error> {
		match value {
			3 => Ok(Self::Accepted),
			4 => Ok(Self::WrongAnswer),
			8 => Ok(Self::CompilationError),
			7 => Ok(Self::RuntimeError),
			5 => Ok(Self::TimeLimitExceeded),
			6 => Ok(Self::MemoryLimitExceeded),
			9 => Ok(Self::InternalError),
			other => anyhow::bail!("unknown submission result: {other}"),
		}
	}
}
