use crate::{prelude::*, proto, repo::Page};
use async_trait::async_trait;

use crate::proto::leetcode::{Submission, SubmissionStatus};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

// pub trait SubmissionRepository {
// 	async fn list(&self, query: SubmissionQuery) -> anyhow::Result<Page<Submission>>;
// 	async fn get(&self, id: &str) -> anyhow::Result<Submission>;
// 	async fn create(&self, submission: CreateSubmission) -> anyhow::Result<Submission>;
// 	async fn update(&self, id: &str, submission: UpdateSubmission) -> anyhow::Result<Submission>;
// 	async fn delete(&self, id: &str) -> anyhow::Result<()>;
// }

#[async_trait]
pub trait SubmissionRepository {
	async fn list(&self, query: SubmissionQuery) -> anyhow::Result<Page<Submission>>;
	async fn get(&self, id: &str) -> anyhow::Result<Submission>;
	async fn create(&self, submission: CreateSubmission) -> anyhow::Result<Submission>;
	async fn update(&self, id: &str, submission: UpdateSubmission) -> anyhow::Result<Submission>;
	async fn delete(&self, id: &str) -> anyhow::Result<()>;
}
