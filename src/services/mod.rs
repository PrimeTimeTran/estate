/// [Goal]
/// This guy should support all platforms and share agnostic api.
///
use crate::{
	app::*,
	model::{submission::*, *},
	proto::types::*,
};

pub mod prelude;
pub use prelude::*;

#[cfg(not(feature = "web"))]
pub use crate::proto::{
	problem_service_server::ProblemService, submission_service_server::SubmissionService,
};

#[derive(Debug)]
pub struct JsonRepo<T> {
	#[cfg(not(feature = "web"))]
	pub path: PathBuf,
	pub _marker: std::marker::PhantomData<T>,
}

#[derive(Clone, Debug)]
pub struct SessionService {
	pub state_service: Arc<StateService>,
}

#[derive(Debug)]
pub struct StateService {
	pub repo: JsonRepo<EstateState>,
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
