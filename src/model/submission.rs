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

// #[derive(Debug, Clone, Default)]
// pub enum SubmissionStatus {
// 	#[default]
// 	Idle,
// 	Submitting,
// 	Running {
// 		id: String,
// 	},
// 	Completed {
// 		id: String,
// 		result: SubmissionResult,
// 	},
// 	Failed {
// 		error: String,
// 	},
// }
// impl TryFrom<i32> for SubmissionResult {
// 	type Error = anyhow::Error;
// 	fn try_from(value: i32) -> Result<Self, Self::Error> {
// 		match value {
// 			0 => Ok(Self::Accepted),
// 			1 => Ok(Self::WrongAnswer),
// 			2 => Ok(Self::CompilationError),
// 			3 => Ok(Self::RuntimeError),
// 			4 => Ok(Self::TimeLimitExceeded),
// 			5 => Ok(Self::MemoryLimitExceeded),
// 			6 => Ok(Self::InternalError),
// 			other => anyhow::bail!("unknown submission result: {other}"),
// 		}
// 	}
// }
