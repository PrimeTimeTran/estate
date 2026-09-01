pub mod problem;
pub use problem::*;
pub mod submission;
pub use submission::*;
pub mod repo;

use crate::{
	model::{submission::*, *},
	prelude::*,
	proto::leetcode::{
		problem_service_server::ProblemService, submission_service_server::SubmissionService,
		types::Problem,
	},
	repo::{problem::*, submission::*},
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct Page<T> {
	pub items: Vec<T>,
	pub page: u32,
	pub page_size: u32,
	pub total: u64,
}

impl<T> Page<T> {
	pub fn page_info(&self) -> PageInfo {
		PageInfo {
			page: self.page as i32,
			page_size: self.page_size as i32,
			total: self.total as i64,
		}
	}
}

pub fn internal_error(error: anyhow::Error) -> Status {
	tracing::error!("{error:#}");
	Status::internal(error.to_string())
}
pub fn page_request(request: Option<PageRequest>) -> Result<PageRequest, Status> {
	request.ok_or_else(|| Status::invalid_argument("page is required"))
}
