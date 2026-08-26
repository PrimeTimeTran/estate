// use crate::{ prelude::*, share::prelude::* };
use crate::{ prelude::* };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(Uuid);

impl Uid {
	pub fn new() -> Self {
		Self(Uuid::new_v4())
	}
}
pub fn now() -> u64 {
	std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}
