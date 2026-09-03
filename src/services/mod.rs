use crate::{
	model::{submission::*, *},
	prelude::*,
};

use crate::proto::types::*;
// use crate::services::prelude::*;

#[derive(Debug)]
pub struct JsonRepo<T> {
	path: PathBuf,
	_marker: std::marker::PhantomData<T>,
}

#[derive(Clone, Debug)]
pub struct SessionService {
	state_service: Arc<StateService>,
}

#[derive(Debug)]
pub struct StateService {
	repo: JsonRepo<EstateState>,
}

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

#[cfg(not(feature = "web"))]
use crate::proto::{
	problem_service_server::ProblemService, submission_service_server::SubmissionService,
};
