use crate::{
	model::{common::Language, *},
	prelude::*,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSolution {
	pub id: String,
	pub problem_id: String,
	pub author_id: String,

	pub title: String,
	pub slug: String,

	pub content: String,
	pub approach: String,

	pub time_complexity: String,
	pub space_complexity: String,

	#[serde(skip)]
	pub code: Vec<ProtoSolutionCode>,

	#[serde(skip)]
	pub status: ProtoSolutionStatus,

	pub view_count: i64,
	pub vote_count: i64,

	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}

impl From<ProtoSolution> for StoredSolution {
	fn from(solution: ProtoSolution) -> Self {
		Self {
			id: solution.id,
			problem_id: solution.problem_id,
			author_id: solution.author_id,
			title: solution.title,
			slug: solution.slug,
			content: solution.content,
			approach: solution.approach,
			time_complexity: solution.time_complexity,
			space_complexity: solution.space_complexity,
			code: solution.code,
			status: solution.status.try_into().unwrap_or_default(),
			view_count: solution.view_count,
			vote_count: solution.vote_count,
			created_at: timestamp_to_datetime(solution.created_at),
			updated_at: timestamp_to_datetime(solution.updated_at),
		}
	}
}

fn timestamp_to_datetime(timestamp: Option<prost_types::Timestamp>) -> Option<DateTime<Utc>> {
	timestamp.and_then(|timestamp| {
		DateTime::<Utc>::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
	})
}
