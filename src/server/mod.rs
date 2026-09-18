/// # External Deps needed by server bin/platform
///
pub use tonic::{Request, Response, Status};

/// # Internal Deps needed by server bin/platform.
///
use crate::prelude::*;

/// # Server modules exposed for use in native/wasm bins.
///
pub mod events;
pub mod fs;
pub mod json;
pub mod native;
pub mod problem;
pub mod repo;
pub mod submission;

pub use crate::proto::{
	problem_service_server::ProblemService, submission_service_server::SubmissionService,
};
pub use crate::server::{events::*, fs::*, problem::*, repo::*, submission::*};

pub fn internal_error(error: anyhow::Error) -> Status {
	tracing::error!("{error:#}");
	Status::internal(error.to_string())
}
pub fn page_request(request: Option<PageRequest>) -> Result<PageRequest, Status> {
	request.ok_or_else(|| Status::invalid_argument("page is required"))
}
