use crate::{prelude::*, proto, repo::Page};
use async_trait::async_trait;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::proto::leetcode::{
	CreateProblemRequest, DeleteProblemRequest, GetProblemRequest, ListProblemsRequest,
	ListProblemsResponse, Problem, UpdateProblemRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodeTemplate {
	language: String,
	code: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Example {
	input: String,
	output: String,
	explanation: String,
}
impl From<StoredExample> for proto::leetcode::Example {
	fn from(value: StoredExample) -> Self {
		Self {
			input: value.input,
			output: value.output,
			explanation: value.explanation,
		}
	}
}

impl From<StoredCodeTemplate> for proto::leetcode::CodeTemplate {
	fn from(value: StoredCodeTemplate) -> Self {
		Self {
			language: value.language,
			source: value.source,
		}
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

	pub created_at: Option<String>,
	pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredExample {
	pub input: String,
	pub output: String,
	pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCodeTemplate {
	pub language: String,
	pub source: String,
}

impl From<StoredProblem> for Problem {
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
			created_at: timestamp(problem.created_at),
			updated_at: timestamp(problem.updated_at),
		}
	}
}
impl From<StoredExample> for Example {
	fn from(example: StoredExample) -> Self {
		Self {
			input: example.input,
			output: example.output,
			explanation: example.explanation,
		}
	}
}

impl From<StoredCodeTemplate> for CodeTemplate {
	fn from(template: StoredCodeTemplate) -> Self {
		Self {
			language: template.language,
			code: template.source,
		}
	}
}

#[async_trait]
pub trait ProblemRepository: Send + Sync {
	async fn list(&self, query: ProblemQuery) -> Result<Page<Problem>>;
	async fn create(&self, problem: CreateProblem) -> Result<Problem>;
	async fn update(&self, id: i64, problem: UpdateProblem) -> Result<Problem>;
	async fn delete(&self, id: i64) -> Result<()>;
	async fn get(&self, id: i64) -> Result<Problem>;
	async fn get_by_slug(&self, slug: &str) -> Result<Problem>;
}

pub struct ProblemQuery {
	pub page: Option<i32>,
	pub page_size: Option<i32>,
	pub difficulty: Option<i32>,
}

pub struct CreateProblem {
	pub title: String,
	pub slug: String,
}

pub struct UpdateProblem {
	pub title: Option<String>,
	pub slug: Option<String>,
}

fn timestamp(value: Option<String>) -> Option<prost_types::Timestamp> {
	value.and_then(|value| {
		chrono::DateTime::parse_from_rfc3339(&value)
			.ok()
			.map(|dt| prost_types::Timestamp {
				seconds: dt.timestamp(),
				nanos: dt.timestamp_subsec_nanos() as i32,
			})
	})
}
