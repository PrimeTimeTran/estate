use tonic::transport::Channel;

use crate::proto::event_service_client::EventServiceClient;

pub mod channel;
pub mod dispatch;
pub mod handler;
pub mod service;

pub use channel::*;
pub use dispatch::*;
pub use handler::*;

pub struct NativeEventTransport {
	pub client: EventServiceClient<Channel>,
}
