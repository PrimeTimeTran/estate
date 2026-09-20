use prost_types::Timestamp;

use crate::{
	prelude::*,
	proto::{
		event_service_server::{EventService as EventServiceTrait, EventServiceServer},
		problem_service_server::ProblemServiceServer,
		submission_service_server::SubmissionServiceServer,
		types as proto_types,
	},
	server::EventBus,
};

#[tonic::async_trait]
impl EventServiceTrait for EventService {
	type SubscribeStream =
		Pin<Box<dyn Stream<Item = Result<proto_types::Event, Status>> + Send + 'static>>;
	async fn subscribe(
		&self,
		_request: Request<proto_types::SubscribeRequest>,
	) -> Result<Response<Self::SubscribeStream>, Status> {
		let rx = self.events.subscribe();
		let stream = BroadcastStream::new(rx).filter_map(|result| match result {
			Ok(event) => {
				// `event` is our INTERNAL app event.
				//
				// Eventually:
				//
				// app_event -> proto_types::Event
				//
				// at this boundary.
				//
				// For now, use a placeholder protobuf event so
				// we can verify the streaming transport itself.
				let proto_event = proto_types::Event {
					id: 0,
					timestamp: 0,
					source: String::from("EventSource::App"),
					kind: format!("{:?}", event.kind),
					payload: format!("{:?}", event),
				};
				Some(Ok(proto_event))
			}
			Err(error) => {
				tracing::error!(?error, "event broadcast failed");
				None
			}
		});
		Ok(Response::new(Box::pin(stream)))
	}
}

// fn event_to_proto(event: Event) -> proto_types::Event {
// 	proto_types::Event {
// 		id: event.id,
// 		kind: event.kind_name(),
// 		payload: serde_json::to_string(&event.kind).expect("EventKind should serialize"),
// 		source: format!("{:?}", event.source),
// 		timestamp: event.timestamp,
// 	}
// }
