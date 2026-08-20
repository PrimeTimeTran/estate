#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(Uuid);

impl Uid {
	pub fn new() -> Self {
		Self(Uuid::now_v7())
	}
}
