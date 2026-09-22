use estate::{
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();
	let builder = ServerBuilder::new();
	let server = builder.build().await?;
	let _events = server.events.clone();
	tokio::spawn(async move {
		let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
		loop {
			interval.tick().await;
			// _events.emit(estate::e::Event::test());
		}
	});

	server.run().await?;

	Ok(())
}

pub struct Server {
	pub problem_service: ProblemService<JsonProblemRepository>,
	pub submission_service: SubmissionService<JsonSubmissionRepository>,
	pub event_service: EventService,

	pub events: EventBus,
}

pub struct ServerBuilder {
	problems_path: &'static str,
	submissions_path: &'static str,
	events: Option<EventBus>,
}

impl ServerBuilder {
	pub fn new() -> Self {
		Self {
			problems_path: estate::data::GRPC_PROBLEMS_PATH,
			submissions_path: estate::data::GRPC_SUBMISSIONS_PATH,
			events: None,
		}
	}

	pub fn events(mut self, events: EventBus) -> Self {
		self.events = Some(events);
		self
	}

	pub async fn build(self) -> Result<Server, Box<dyn std::error::Error>> {
		let problems = JsonProblemRepository::new(self.problems_path);
		let submissions = JsonSubmissionRepository::new(self.submissions_path);
		let page = problems
			.list(ProblemQuery {
				page: Some(0),
				page_size: Some(1),
				difficulty: None,
			})
			.await?;

		println!("📚 Problems available: {}", page.total);

		let problem_service = ProblemService::new(problems);
		let submission_service = SubmissionService::new(submissions);

		let events = self.events.unwrap_or_default();
		let event_service = EventService::new(events.clone());

		Ok(Server {
			problem_service,
			submission_service,
			event_service,
			events,
		})
	}
}

impl Server {
	pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
		let addr = estate::data::GRPC_SOCKET.parse()?;
		println!("API listening on {addr}");
		tonic::transport::Server::builder()
			.add_service(ProblemServiceServer::new(self.problem_service))
			.add_service(SubmissionServiceServer::new(self.submission_service))
			.add_service(EventServiceServer::new(self.event_service))
			.serve(addr)
			.await?;

		Ok(())
	}
}
