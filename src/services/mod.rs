pub mod problem;
pub use problem::*;
pub mod submission;
pub use submission::*;
pub mod repo;

use crate::leetcode::PageRequest;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use tonic::{Request, Response, Status};

pub fn internal_error(error: anyhow::Error) -> Status {
	tracing::error!("{error:#}");
	Status::internal(error.to_string())
}
pub fn page_request(request: Option<PageRequest>) -> Result<PageRequest, Status> {
	request.ok_or_else(|| Status::invalid_argument("page is required"))
}
pub fn parse_timestamp(value: Option<String>) -> Option<prost_types::Timestamp> {
	value.and_then(|value| {
		value
			.parse::<chrono::DateTime<chrono::Utc>>()
			.ok()
			.map(|dt| prost_types::Timestamp {
				seconds: dt.timestamp(),
				nanos: dt.timestamp_subsec_nanos() as i32,
			})
	})
}
pub fn timestamp(value: Option<String>) -> Option<prost_types::Timestamp> {
	value.and_then(|value| {
		chrono::DateTime::parse_from_rfc3339(&value)
			.ok()
			.map(|dt| prost_types::Timestamp {
				seconds: dt.timestamp(),
				nanos: dt.timestamp_subsec_nanos() as i32,
			})
	})
}
