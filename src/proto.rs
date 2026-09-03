// VSCode does not recognize this paths via cmd+click.
// With default = ["native"] in Cargo.toml Zed does.
// - [x] Native builds.
// cargo build --bin native --features native
// - [ ] WASM builds.
pub mod types {
	include!(concat!(env!("OUT_DIR"), "/leetcode.types.rs"));
}
#[cfg(not(target_arch = "wasm32"))]
tonic::include_proto!("leetcode");
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
