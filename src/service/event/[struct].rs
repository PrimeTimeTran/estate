use crate::prelude::*;

// Subscribes to local events, transforms into a protobuf
pub struct EventService {
	pub events: EventBus,
}
