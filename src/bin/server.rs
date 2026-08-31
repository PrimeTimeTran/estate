use std::net::SocketAddr;

use estate::proto::leetcode::problem_service_server::ProblemServiceServer;
use estate::repo::json::problem::JsonProblemRepository;
use estate::services::problem::ProblemServiceImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr: SocketAddr = "127.0.0.1:50051".parse()?;
	// let repository = JsonProblemRepository::new("./problems");
	let repository = JsonProblemRepository::new("src/data/problems");
	let problem = repository.load("two-sum").await?;
	println!("problem {:?}", problem);
	let service = ProblemServiceImpl::new(repository);
	tonic::transport::Server::builder()
		.add_service(ProblemServiceServer::new(service))
		.serve(addr)
		.await?;
	Ok(())
}
