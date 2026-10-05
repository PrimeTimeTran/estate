pub mod channel;
pub mod dispatch;
pub mod handler;
pub mod service;

pub use crate::server::event::channel::*;
pub use crate::server::event::dispatch::*;
pub use crate::server::event::handler::*;

use crate::proto::event_service_client::EventServiceClient;
use tonic::{transport::Channel};

pub struct NativeEventTransport {
	pub client: EventServiceClient<Channel>,
}
