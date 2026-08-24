use crate::{prelude::*, shared::prelude::*};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(Uuid);

impl Uid {
	pub fn new() -> Self {
		Self(Uuid::new_v4())
	}
}
