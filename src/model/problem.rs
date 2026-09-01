use crate::{
	model::{ProtoProblem, common::Language, *},
	prelude::*,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Hash)]
pub struct StoredProblem {
	pub id: i64,
	pub number: i32,
	pub title: String,
	pub slug: String,
	pub description: String,
	pub difficulty: i32,
	pub tags: Vec<String>,
	pub examples: Vec<StoredExample>,
	pub constraints: Vec<String>,
	pub code_templates: Vec<StoredCodeTemplate>,
	pub is_published: bool,
	pub created_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
}
impl From<StoredProblem> for ProtoProblem {
	fn from(problem: StoredProblem) -> Self {
		Self {
			id: problem.id.to_string(),
			number: problem.number,
			title: problem.title,
			slug: problem.slug,
			description: problem.description,
			difficulty: problem.difficulty,
			tags: problem.tags,
			examples: problem.examples.into_iter().map(Into::into).collect(),
			constraints: problem.constraints,
			code_templates: problem.code_templates.into_iter().map(Into::into).collect(),
			is_published: problem.is_published,
			created_at: to_timestamp(problem.created_at.as_ref()),
			updated_at: to_timestamp(problem.updated_at.as_ref()),
		}
	}
}
impl TryFrom<ProtoProblem> for StoredProblem {
	type Error = anyhow::Error;

	fn try_from(problem: ProtoProblem) -> Result<Self, Self::Error> {
		Ok(Self {
			id: problem.id.parse().unwrap_or_default(),
			number: problem.number,
			title: problem.title,
			slug: problem.slug,
			description: problem.description,
			difficulty: problem.difficulty,
			tags: problem.tags,
			examples: problem.examples.into_iter().map(Into::into).collect(),
			constraints: problem.constraints,
			code_templates: problem
				.code_templates
				.into_iter()
				.map(StoredCodeTemplate::try_from)
				.collect::<Result<Vec<_>, _>>()?,
			is_published: problem.is_published,
			created_at: None,
			updated_at: None,
		})
	}
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct StoredCodeTemplate {
	pub language: Language,
	pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct StoredExample {
	pub input: String,
	pub output: String,
	pub explanation: String,
}
impl From<StoredExample> for Example {
	fn from(value: StoredExample) -> Self {
		Self {
			input: value.input,
			output: value.output,
			explanation: value.explanation,
		}
	}
}
impl From<StoredCodeTemplate> for CodeTemplate {
	fn from(value: StoredCodeTemplate) -> Self {
		Self {
			language: ProtoLanguage::from(value.language) as i32,
			source: value.source,
		}
	}
}
impl From<Example> for StoredExample {
	fn from(value: Example) -> Self {
		Self {
			input: value.input,
			output: value.output,
			explanation: value.explanation,
		}
	}
}
impl TryFrom<CodeTemplate> for StoredCodeTemplate {
	type Error = anyhow::Error;

	fn try_from(value: CodeTemplate) -> Result<Self, Self::Error> {
		let language = ProtoLanguage::try_from(value.language)?;

		Ok(Self {
			language: Language::try_from(language)?,
			source: value.source,
		})
	}
}
