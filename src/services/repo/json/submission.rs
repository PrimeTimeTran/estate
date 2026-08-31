use crate::proto::leetcode::{Submission, SubmissionStatus};
use crate::{
	repo::submission::{CreateSubmission, SubmissionQuery, SubmissionRepository, UpdateSubmission},
	services::*,
};
pub struct JsonSubmissionRepository {
	path: PathBuf,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoredSubmission {
	pub id: String,
	pub user_id: String,
	pub problem_id: String,
	pub source: String,
	pub language: String,
	pub status: i32,
	pub runtime_ms: Option<i64>,
	pub memory_bytes: Option<i64>,
	pub error: Option<String>,
	pub tests_passed: Option<i32>,
	pub tests_total: Option<i32>,
	pub created_at: Option<chrono::DateTime<chrono::Utc>>,
	pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
use std::time::SystemTime;
impl From<StoredSubmission> for Submission {
	fn from(value: StoredSubmission) -> Self {
		Self {
			id: value.id,
			user_id: value.user_id,
			problem_id: value.problem_id,
			source: value.source,
			language: value.language,
			status: value.status,
			runtime_ms: value.runtime_ms,
			memory_bytes: value.memory_bytes,
			error: value.error,
			tests_passed: value.tests_passed,
			tests_total: value.tests_total,
			created_at: value
				.created_at
				.map(|dt| prost_types::Timestamp::from(std::time::SystemTime::from(dt))),
			updated_at: value
				.updated_at
				.map(|dt| prost_types::Timestamp::from(std::time::SystemTime::from(dt))),
		}
	}
}
impl JsonSubmissionRepository {
	pub fn new(path: impl Into<PathBuf>) -> Self {
		Self { path: path.into() }
	}
	fn submission_path(&self, id: &str) -> PathBuf {
		self.path.join(format!("{id}.json"))
	}
	pub async fn load(&self, id: &str) -> Result<StoredSubmission> {
		let path = self.submission_path(id);
		let contents = tokio::fs::read_to_string(&path)
			.await
			.with_context(|| format!("failed to read submission: {}", path.display()))?;
		serde_json::from_str(&contents)
			.with_context(|| format!("failed to parse submission: {}", path.display()))
	}
	pub async fn save(&self, submission: &StoredSubmission) -> Result<()> {
		tokio::fs::create_dir_all(&self.path)
			.await
			.with_context(|| {
				format!(
					"failed to create submission directory: {}",
					self.path.display()
				)
			})?;
		let path = self.submission_path(&submission.id);
		let contents = serde_json::to_string_pretty(submission)?;
		tokio::fs::write(&path, contents)
			.await
			.with_context(|| format!("failed to write submission: {}", path.display()))?;
		Ok(())
	}
}
#[async_trait]
impl SubmissionRepository for JsonSubmissionRepository {
	async fn list(&self, query: SubmissionQuery) -> Result<Page<Submission>> {
		let page = query.page.unwrap_or(0).max(0) as u32;
		let page_size = query.page_size.unwrap_or(20).max(1) as u32;
		let mut submissions = Vec::new();
		let mut entries = match tokio::fs::read_dir(&self.path).await {
			Ok(entries) => entries,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(Page {
					items: Vec::new(),
					page,
					page_size,
					total: 0,
				});
			}
			Err(error) => return Err(error.into()),
		};
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.extension().and_then(|x| x.to_str()) != Some("json") {
				continue;
			}
			let contents = tokio::fs::read_to_string(&path)
				.await
				.with_context(|| format!("failed to read {}", path.display()))?;
			let stored: StoredSubmission = serde_json::from_str(&contents)
				.with_context(|| format!("failed to parse {}", path.display()))?;
			if query
				.user_id
				.as_deref()
				.is_some_and(|user_id| stored.user_id != user_id)
			{
				continue;
			}
			if query
				.problem_id
				.as_deref()
				.is_some_and(|problem_id| stored.problem_id != problem_id)
			{
				continue;
			}
			if query
				.status
				.is_some_and(|status| stored.status != status as i32)
			{
				continue;
			}
			if query
				.language
				.as_deref()
				.is_some_and(|language| stored.language != language)
			{
				continue;
			}
			submissions.push(stored.into());
		}
		let total = submissions.len() as u64;
		let start = (page * page_size) as usize;
		let end = (start + page_size as usize).min(submissions.len());
		let items = submissions.get(start..end).unwrap_or_default().to_vec();
		Ok(Page {
			items,
			page,
			page_size,
			total,
		})
	}
	async fn create(&self, submission: CreateSubmission) -> Result<Submission> {
		let stored = StoredSubmission {
			id: self.next_id().await?,
			user_id: submission.user_id,
			problem_id: submission.problem_id,
			source: submission.source,
			language: submission.language,
			..Default::default()
		};
		self.save(&stored).await?;
		Ok(stored.into())
	}
	async fn update(&self, id: &str, update: UpdateSubmission) -> Result<Submission> {
		let mut submission = self.find_by_id(id).await?;
		if let Some(source) = update.source {
			submission.source = source;
		}
		if let Some(language) = update.language {
			submission.language = language;
		}
		self.save(&submission).await?;
		Ok(submission.into())
	}
	async fn delete(&self, id: &str) -> Result<()> {
		let submission = self.find_by_id(id).await?;
		let path = self.submission_path(&submission.id);
		tokio::fs::remove_file(&path)
			.await
			.with_context(|| format!("failed to delete {}", path.display()))?;
		Ok(())
	}
	async fn get(&self, id: &str) -> Result<Submission> {
		Ok(self.find_by_id(id).await?.into())
	}
}
impl JsonSubmissionRepository {
	async fn find_by_id(&self, id: &str) -> Result<StoredSubmission> {
		let path = self.submission_path(id);
		let contents = tokio::fs::read_to_string(&path)
			.await
			.with_context(|| format!("failed to read submission: {}", path.display()))?;
		serde_json::from_str(&contents)
			.with_context(|| format!("failed to parse submission: {}", path.display()))
	}
	async fn next_id(&self) -> Result<String> {
		let mut entries = match tokio::fs::read_dir(&self.path).await {
			Ok(entries) => entries,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok("1".to_string());
			}
			Err(error) => return Err(error.into()),
		};
		let mut max_id = 0i64;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.extension().and_then(|x| x.to_str()) != Some("json") {
				continue;
			}
			let contents = tokio::fs::read_to_string(&path).await?;
			let submission: StoredSubmission = serde_json::from_str(&contents)?;
			let id = submission.id.parse::<i64>().unwrap_or(0);
			max_id = max_id.max(id);
		}
		Ok((max_id + 1).to_string())
	}
}