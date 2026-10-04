use crate::{
	e::{EventBus, EventService},
	prelude::*,
	proto::{
		event_service_server::EventServiceServer
	},
};

pub async fn run_server(cancel: CancellationToken) -> anyhow::Result<()> {
	tracing::info!("🚀 Estate gRPC server starting");

	let builder = ServerBuilder::new();
	let server = builder.build().await?;

	let addr = crate::data::GRPC_SOCKET.parse::<std::net::SocketAddr>()?;

	tracing::info!(%addr, "🌐 Estate gRPC server listening");

	tokio::select! {
		result = server.run() => {
			result?;
		}

		_ = cancel.cancelled() => {
			tracing::info!("🛑 Estate gRPC server cancellation requested");
		}
	}

	Ok(())
}
pub struct Server {
	pub event_service: EventService,

	pub events: EventBus,
}

pub struct ServerBuilder {
	events: Option<EventBus>,
}

impl ServerBuilder {
	pub fn new() -> Self {
		Self {
			events: None,
		}
	}

	pub fn events(mut self, events: EventBus) -> Self {
		self.events = Some(events);
		self
	}

	pub async fn build(self) -> anyhow::Result<Server> {
		let events = self.events.unwrap_or_default();
		let event_service = EventService::new(events.clone());
		Ok(Server {
			event_service,
			events,
		})
	}
}

impl Server {
	pub async fn run(self) -> anyhow::Result<()> {
		let addr = crate::data::GRPC_SOCKET.parse()?;
		tracing::info!(%addr, "🌐 Estate gRPC server listening");
		tonic::transport::Server::builder()
			.add_service(EventServiceServer::new(self.event_service))
			.serve(addr)
			.await?;
		Ok(())
	}
}
