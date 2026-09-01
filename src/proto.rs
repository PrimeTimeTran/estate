pub mod leetcode {
	pub mod types {
		include!(concat!(env!("OUT_DIR"), "/leetcode.types.rs"));
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

// #[cfg(not(target_arch = "wasm32"))]
// pub async fn client() -> anyhow::Result<NativeClient> {
//     // tonic
// }
// #[cfg(target_arch = "wasm32")]
// pub async fn client() -> anyhow::Result<WebClient> {
//     // gRPC-Web/browser transport
// }
