use std::net::SocketAddr;

use estate::{
	proto::{
		problem_service_server::ProblemServiceServer,
		submission_service_server::SubmissionServiceServer,
	},
	server::{
		json::{problem::JsonProblemRepository, submission::JsonSubmissionRepository},
		problem::ProblemServiceImpl,
		problem::{ProblemQuery, ProblemRepository},
		submission::SubmissionServiceImpl,
	},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();
	let addr: SocketAddr = estate::data::GRPC_SOCKET.parse()?;
	// Repositories
	let problem_repository = JsonProblemRepository::new(estate::data::GRPC_PROBLEMS_PATH);
	let page = problem_repository
		.list(ProblemQuery {
			page: Some(0),
			page_size: Some(1),
			difficulty: None,
		})
		.await?;
	println!("📚 Problems available: {}", page.total);
	let submission_repository = JsonSubmissionRepository::new(estate::data::GRPC_SUBMISSIONS_PATH);

	let problem_service = ProblemServiceImpl::new(problem_repository);

	let submission_service = SubmissionServiceImpl::new(submission_repository);

	println!("API listening on {addr}");

	tonic::transport::Server::builder()
		.add_service(ProblemServiceServer::new(problem_service))
		.add_service(SubmissionServiceServer::new(submission_service))
		.serve(addr)
		.await?;

	Ok(())
}

pub struct App {
	pub problems: JsonProblemRepository,
	pub submissions: JsonSubmissionRepository,
}
