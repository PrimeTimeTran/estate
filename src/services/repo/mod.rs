use crate::leetcode::PageInfo;

pub mod json;
pub mod problem;
pub mod submission;

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
