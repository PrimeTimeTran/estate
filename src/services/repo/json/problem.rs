use crate::proto::leetcode::{types::Problem, *};

use crate::services::*;

pub struct JsonProblemRepository {
	path: PathBuf,
}
impl JsonProblemRepository {
	pub fn new(path: impl Into<PathBuf>) -> Self {
		Self { path: path.into() }
	}
	fn problem_path(&self, slug: &str) -> PathBuf {
		self.path.join(format!("{slug}.json"))
	}
	pub async fn load(&self, slug: &str) -> Result<StoredProblem> {
		let path = self.problem_path(slug);
		let contents = tokio::fs::read_to_string(&path)
			.await
			.with_context(|| format!("failed to read problem: {}", path.display()))?;
		serde_json::from_str(&contents)
			.with_context(|| format!("failed to parse problem: {}", path.display()))
	}
	async fn load_matching(&self, query: &ProblemQuery) -> Result<Vec<Problem>> {
		let mut problems = Vec::new();

		let mut entries = tokio::fs::read_dir(&self.path)
			.await
			.with_context(|| format!("failed to read {}", self.path.display()))?;

		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();

			if path.extension().and_then(|x| x.to_str()) != Some("json") {
				continue;
			}

			let contents = tokio::fs::read_to_string(&path)
				.await
				.with_context(|| format!("failed to read {}", path.display()))?;

			let stored: StoredProblem = serde_json::from_str(&contents)
				.with_context(|| format!("failed to parse {}", path.display()))?;

			if query
				.difficulty
				.is_some_and(|d| stored.difficulty != d as i32)
			{
				continue;
			}

			problems.push(stored.into());
		}

		Ok(problems)
	}
	pub async fn save(&self, problem: &StoredProblem) -> Result<()> {
		tokio::fs::create_dir_all(&self.path)
			.await
			.with_context(|| {
				format!(
					"failed to create problem directory: {}",
					self.path.display()
				)
			})?;
		let path = self.problem_path(&problem.slug);
		let contents = serde_json::to_string_pretty(problem)?;
		tokio::fs::write(&path, contents)
			.await
			.with_context(|| format!("failed to write problem: {}", path.display()))?;
		Ok(())
	}
}
#[async_trait]
impl ProblemRepository for JsonProblemRepository {
	async fn list(&self, query: ProblemQuery) -> Result<Page<Problem>> {
		let page = query.page.unwrap_or(0).max(0) as u32;
		let page_size = query.page_size.unwrap_or(20).max(1) as u32;
		let mut problems = Vec::new();
		let mut entries = tokio::fs::read_dir(&self.path)
			.await
			.with_context(|| format!("failed to read {}", self.path.display()))?;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.extension().and_then(|x| x.to_str()) != Some("json") {
				continue;
			}
			let contents = tokio::fs::read_to_string(&path)
				.await
				.with_context(|| format!("failed to read {}", path.display()))?;
			let stored: StoredProblem = serde_json::from_str(&contents)
				.with_context(|| format!("failed to parse {}", path.display()))?;
			if query
				.difficulty
				.is_some_and(|d| stored.difficulty != d as i32)
			{
				continue;
			}
			problems.push(stored.into());
		}
		let total = problems.len() as u64;
		let start = (page * page_size) as usize;
		let end = (start + page_size as usize).min(problems.len());
		let items = problems.get(start..end).unwrap_or_default().to_vec();
		Ok(Page {
			items,
			page,
			page_size,
			total,
		})
	}
	async fn create(&self, problem: CreateProblem) -> Result<Problem> {
		let stored = StoredProblem {
			id: self.next_id().await?,
			title: problem.title,
			slug: problem.slug,
			..Default::default()
		};
		self.save(&stored).await?;
		Ok(stored.into())
	}
	async fn update(&self, id: i64, update: UpdateProblem) -> Result<Problem> {
		let mut problem = self.find_by_id(id).await?;
		let old_slug = problem.slug.clone();
		if let Some(title) = update.title {
			problem.title = title;
		}
		if let Some(slug) = update.slug {
			problem.slug = slug;
		}
		if problem.slug != old_slug {
			let old_path = self.problem_path(&old_slug);
			if tokio::fs::try_exists(&old_path).await? {
				tokio::fs::remove_file(&old_path).await?;
			}
		}
		self.save(&problem).await?;
		Ok(problem.into())
	}
	async fn delete(&self, id: i64) -> Result<()> {
		let problem = self.find_by_id(id).await?;
		let path = self.problem_path(&problem.slug);
		tokio::fs::remove_file(&path)
			.await
			.with_context(|| format!("failed to delete {}", path.display()))?;
		Ok(())
	}
	async fn get(&self, id: i64) -> Result<Problem> {
		Ok(self.find_by_id(id).await?.into())
	}
	async fn get_by_slug(&self, slug: &str) -> Result<Problem> {
		Ok(self.load(slug).await?.into())
	}

	async fn sample_problem(&self, query: ProblemQuery) -> Result<Problem> {
		use rand::seq::IndexedRandom;

		let mut problems = Vec::new();

		let mut entries = tokio::fs::read_dir(&self.path)
			.await
			.with_context(|| format!("failed to read {}", self.path.display()))?;

		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();

			if path.extension().and_then(|x| x.to_str()) != Some("json") {
				continue;
			}

			let contents = tokio::fs::read_to_string(&path)
				.await
				.with_context(|| format!("failed to read {}", path.display()))?;

			let stored: StoredProblem = serde_json::from_str(&contents)
				.with_context(|| format!("failed to parse {}", path.display()))?;
			// stored.difficulty
			if query
				.difficulty
				.is_some_and(|difficulty| stored.difficulty != difficulty as i32)
			{
				continue;
			}

			problems.push(Problem::from(stored));
		}

		let problem = problems
			.choose(&mut rand::rng())
			.cloned()
			.ok_or_else(|| anyhow::anyhow!("no problems matched the query"))?;

		Ok(problem)
	}
}
impl JsonProblemRepository {
	async fn find_by_id(&self, id: i64) -> Result<StoredProblem> {
		let mut entries = tokio::fs::read_dir(&self.path)
			.await
			.with_context(|| format!("failed to read {}", self.path.display()))?;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.extension().and_then(|x| x.to_str()) != Some("json") {
				continue;
			}
			let contents = tokio::fs::read_to_string(&path)
				.await
				.with_context(|| format!("failed to read {}", path.display()))?;
			let problem: StoredProblem = serde_json::from_str(&contents)
				.with_context(|| format!("failed to parse {}", path.display()))?;
			if problem.id == id {
				return Ok(problem);
			}
		}
		anyhow::bail!("problem {id} not found")
	}
	async fn next_id(&self) -> Result<i64> {
		let mut entries = match tokio::fs::read_dir(&self.path).await {
			Ok(entries) => entries,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(1);
			}
			Err(error) => return Err(error.into()),
		};
		let mut max_id = 0;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			if path.extension().and_then(|x| x.to_str()) != Some("json") {
				continue;
			}
			let contents = tokio::fs::read_to_string(&path).await?;
			let problem: StoredProblem = serde_json::from_str(&contents)?;
			max_id = max_id.max(problem.id);
		}
		Ok(max_id + 1)
	}
}
