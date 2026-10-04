use crate::{
	e::{EventBus, EventService},
	modules::server::{
		json::{problem::JsonProblemRepository, submission::JsonSubmissionRepository},
		problem::{ProblemQuery, ProblemService},
		submission::SubmissionService,
	},
	prelude::*,
	proto::{
		event_service_server::EventServiceServer, problem_service_server::ProblemServiceServer,
		submission_service_server::SubmissionServiceServer,
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
	// pub problem_service: ProblemService<JsonProblemRepository>,
	// pub submission_service: SubmissionService<JsonSubmissionRepository>,
	pub event_service: EventService,

	pub events: EventBus,
}

pub struct ServerBuilder {
	// problems_path: &'static str,
	// submissions_path: &'static str,
	events: Option<EventBus>,
}

impl ServerBuilder {
	pub fn new() -> Self {
		Self {
			// problems_path: crate::data::GRPC_PROBLEMS_PATH,
			// submissions_path: crate::data::GRPC_SUBMISSIONS_PATH,
			events: None,
		}
	}

	pub fn events(mut self, events: EventBus) -> Self {
		self.events = Some(events);
		self
	}

	pub async fn build(self) -> anyhow::Result<Server> {
		// let problems = JsonProblemRepository::new(self.problems_path);
		// let submissions = JsonSubmissionRepository::new(self.submissions_path);
// 		let page = problems
// 			.list(ProblemQuery {
// 				page: Some(0),
// 				page_size: Some(1),
// 				difficulty: None,
// 			})
// 			.await?;
// 
// 		println!("📚 Problems available: {}", page.total);
// 
// 		let problem_service = ProblemService::new(problems);
// 		let submission_service = SubmissionService::new(submissions);

		let events = self.events.unwrap_or_default();
		let event_service = EventService::new(events.clone());

		Ok(Server {
			// problem_service,
			// submission_service,
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
			// .add_service(ProblemServiceServer::new(self.problem_service))
			// .add_service(SubmissionServiceServer::new(self.submission_service))
			.add_service(EventServiceServer::new(self.event_service))
			.serve(addr)
			.await?;

		Ok(())
	}
}
