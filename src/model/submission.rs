use crate::{
	model::{Language, ProtoSubmissionStatus},
	prelude::*,
};

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct StoredSubmission {
	pub id: i64,
	pub user_id: i64,
	pub problem_id: i64,

	pub source: String,
	pub language: Language,

	pub status: SubmissionStatus,

	pub runtime_ms: Option<i64>,
	pub memory_bytes: Option<i64>,

	pub error: Option<String>,

	pub tests_passed: Option<i32>,
	pub tests_total: Option<i32>,

	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStatus {
	#[default]
	Pending,
	Running,
	Accepted,
	WrongAnswer,
	TimeLimitExceeded,
	MemoryLimitExceeded,
	RuntimeError,
	CompilationError,
	InternalError,
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct SubmissionState {
	pub status: SubmissionStatus,
	pub id: Option<String>,
	pub error: Option<String>,
}

pub struct CreateSubmission {
	pub user_id: String,
	pub problem_id: String,
	pub source: String,
	pub language: Language,
}
pub struct UpdateSubmission {
	pub source: Option<String>,
	pub language: Language,
}
pub struct SubmissionQuery {
	pub page: Option<i32>,
	pub page_size: Option<i32>,
	pub user_id: Option<String>,
	pub problem_id: Option<String>,
	pub status: Option<SubmissionStatus>,
	pub language: Option<Language>,
}

impl From<SubmissionStatus> for ProtoSubmissionStatus {
	fn from(value: SubmissionStatus) -> Self {
		match value {
			SubmissionStatus::Pending => Self::Pending,
			SubmissionStatus::Running => Self::Running,
			SubmissionStatus::Accepted => Self::Accepted,
			SubmissionStatus::WrongAnswer => Self::WrongAnswer,
			SubmissionStatus::TimeLimitExceeded => Self::TimeLimitExceeded,
			SubmissionStatus::MemoryLimitExceeded => Self::MemoryLimitExceeded,
			SubmissionStatus::RuntimeError => Self::RuntimeError,
			SubmissionStatus::CompilationError => Self::CompilationError,
			SubmissionStatus::InternalError => Self::InternalError,
		}
	}
}

impl TryFrom<ProtoSubmissionStatus> for SubmissionStatus {
	type Error = anyhow::Error;

	fn try_from(value: ProtoSubmissionStatus) -> Result<Self, Self::Error> {
		match value {
			ProtoSubmissionStatus::Pending => Ok(Self::Pending),
			ProtoSubmissionStatus::Running => Ok(Self::Running),
			ProtoSubmissionStatus::Accepted => Ok(Self::Accepted),
			ProtoSubmissionStatus::WrongAnswer => Ok(Self::WrongAnswer),
			ProtoSubmissionStatus::TimeLimitExceeded => Ok(Self::TimeLimitExceeded),
			ProtoSubmissionStatus::MemoryLimitExceeded => Ok(Self::MemoryLimitExceeded),
			ProtoSubmissionStatus::RuntimeError => Ok(Self::RuntimeError),
			ProtoSubmissionStatus::CompilationError => Ok(Self::CompilationError),
			ProtoSubmissionStatus::InternalError => Ok(Self::InternalError),
			ProtoSubmissionStatus::Unspecified => {
				anyhow::bail!("submission status was unspecified")
			}
		}
	}
}

pub struct SubmissionExecution {
	pub status: SubmissionStatus,
	pub runtime_ms: Option<i64>,
	pub memory_bytes: Option<i64>,
	pub tests_passed: Option<i32>,
	pub tests_total: Option<i32>,
	pub error: Option<String>,
}
