use estate::{
	proto::{
		problem_service_server::ProblemServiceServer,
		submission_service_server::SubmissionServiceServer,
	},
	server::{
		json::{problem::JsonProblemRepository, submission::JsonSubmissionRepository},
		problem::{ProblemQuery, ProblemService},
		submission::SubmissionService,
	},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();
	let addr = estate::data::GRPC_SOCKET.parse()?;

	// Repositories
	let problem_repo = JsonProblemRepository::new(estate::data::GRPC_PROBLEMS_PATH);
	let page = problem_repo
		.list(ProblemQuery {
			page: Some(0),
			page_size: Some(1),
			difficulty: None,
		})
		.await?;
	println!("📚 Problems available: {}", page.total);

	let submission_repo = JsonSubmissionRepository::new(estate::data::GRPC_SUBMISSIONS_PATH);
	let problem_service = ProblemService::new(problem_repo);
	let submission_service = SubmissionService::new(submission_repo);
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
