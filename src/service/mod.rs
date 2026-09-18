/// ## Service
///
/// These need to be available outside of Server because client/wasm code
/// also depends on the availability of the page helpers as well.
///
use crate::prelude::*;

impl<T> Page<T> {
	pub fn page_info(&self) -> PageInfo {
		PageInfo {
			page: self.page as i32,
			page_size: self.page_size as i32,
			total: self.total as i64,
		}
	}
}

#[derive(Debug)]
pub struct JsonRepo<T> {
	#[cfg(not(feature = "web"))]
	pub path: PathBuf,
	pub _marker: std::marker::PhantomData<T>,
}

#[derive(Debug)]
pub struct Page<T> {
	pub items: Vec<T>,
	pub page: u32,
	pub page_size: u32,
	pub total: u64,
}

#[derive(Clone, Debug)]
pub struct SessionService {
	pub state_service: Arc<StateService>,
}

#[derive(Debug)]
pub struct StateService {
	pub repo: JsonRepo<EstateState>,
}
