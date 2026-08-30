use std::net::SocketAddr;

use estate::app::services::problem::ProblemServiceImpl;
use estate::proto::leetcode::problem_service_server::ProblemServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr: SocketAddr = "127.0.0.1:50051".parse()?;

	println!("🚀 gRPC server listening on {addr}");

	tonic::transport::Server::builder()
		.add_service(ProblemServiceServer::new(ProblemServiceImpl::default()))
		.serve(addr)
		.await?;

	Ok(())
}
