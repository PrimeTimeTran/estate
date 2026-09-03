pub mod event;
pub mod json;
pub mod native;
pub mod problem;
pub mod submission;
pub use event::*;
pub use json::*;
pub use native::*;
pub use problem::*;
pub use submission::*;

pub use crate::proto::types::*;
pub use crate::server::native::*;

use tonic::{Request, Response, Status};

pub fn internal_error(error: anyhow::Error) -> Status {
	tracing::error!("{error:#}");
	Status::internal(error.to_string())
}

pub fn page_request(request: Option<PageRequest>) -> Result<PageRequest, Status> {
	request.ok_or_else(|| Status::invalid_argument("page is required"))
}
