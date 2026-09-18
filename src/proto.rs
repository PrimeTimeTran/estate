//! ## Protobuf Type defs
//! 
//! Three different platforms must use these type defs
//! 
//! Native, Wasm (Web), Server
//! 
//! So the build system is kinda complicated.
//! 
pub mod types {
	include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

#[cfg(not(target_arch = "wasm32"))]
tonic::include_proto!("prototypes");

#[cfg(not(target_arch = "wasm32"))]
pub async fn client()
-> anyhow::Result<problem_service_client::ProblemServiceClient<tonic::transport::Channel>> {
	Ok(problem_service_client::ProblemServiceClient::connect(crate::SERVER_URL).await?)
}
