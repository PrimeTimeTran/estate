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
