use std::net::SocketAddr;

use tonic::{Request, Response, Status};

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

use estate::proto::leetcode::{
	CreateProblemRequest, DeleteProblemRequest, Empty, GetProblemRequest, ListProblemsRequest,
	ListProblemsResponse, PageInfo, Problem, UpdateProblemRequest,
	problem_service_server::ProblemService,
};

#[derive(Default)]
struct ProblemServiceImpl;

#[tonic::async_trait]
impl ProblemService for ProblemServiceImpl {
	async fn list_problems(
		&self,
		_request: Request<ListProblemsRequest>,
	) -> Result<Response<ListProblemsResponse>, Status> {
		Ok(Response::new(ListProblemsResponse {
			page: Some(PageInfo {
				page: 0,
				page_size: 0,
				total: 0,
			}),
			problems: vec![],
			// Add any other required fields from your proto here.
		}))
	}
	async fn create_problem(
		&self,
		request: Request<CreateProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		todo!("create")
	}
	async fn update_problem(
		&self,
		request: Request<UpdateProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		todo!("update")
	}
	async fn delete_problem(
		&self,
		request: Request<DeleteProblemRequest>,
	) -> Result<Response<Empty>, Status> {
		todo!("delete_problem")
	}
	async fn get_problem(
		&self,
		request: Request<GetProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		todo!("get_problem")
	}
}
