// pub mod leetcode {
// 	pub mod types {
// 		tonic::include_proto!("leetcode.types");
// 	}
// 	#[cfg(not(target_arch = "wasm32"))]
// 	tonic::include_proto!("leetcode");
// }
pub mod leetcode {
	pub mod types {
		tonic::include_proto!("leetcode.types");
	}
	#[cfg(not(target_arch = "wasm32"))]
	tonic::include_proto!("leetcode");
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn client()
-> anyhow::Result<leetcode::problem_service_client::ProblemServiceClient<tonic::transport::Channel>>
{
	Ok(leetcode::problem_service_client::ProblemServiceClient::connect(crate::SERVER_URL).await?)
}
