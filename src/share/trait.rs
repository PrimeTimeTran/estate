use crate::{ prelude::*, share::* };

pub trait Runtime: Clone + Send + Sync {
	fn emit(&self, event: Event);
	fn start_dispatcher(self: &Arc<Self>);
}
