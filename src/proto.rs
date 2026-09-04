/// VSCode does not recognize this paths via cmd+click.
/// Three different bins must build using proto defs found here.
/// - Server:
/// - Native:
/// - Wasm:
///
/// ## [Native]
/// - Given default = ["native"] in Cargo.toml Zed does.
/// - [x] Native builds.
///
/// ### [Build]
/// cargo build --bin native --features native
///
/// ## [Web]
/// - Given default = ["web"] in Cargo.toml
/// - [ ] WASM builds.
///
/// ### [Build]
/// cargo build --bin web --no-default-features --features="web" --target wasm32-unknown-unknown
///
///
/// WIP: Add Native & Web builds without breaking each other.
/// - Native built. Web didn't
/// - Web built. Native didn't.
/// - Web & Native built. Server didnt'
pub mod types {
	include!(concat!(env!("OUT_DIR"), "/types.rs"));
}
#[cfg(not(target_arch = "wasm32"))]
tonic::include_proto!("leetcode");
// pub mod leetcode {
// 	pub mod types {
// 		include!(concat!(env!("OUT_DIR"), "/types.rs"));
// 	}
// 	#[cfg(not(target_arch = "wasm32"))]
// 	tonic::include_proto!("leetcode");
// }

#[cfg(not(target_arch = "wasm32"))]
pub async fn client()
-> anyhow::Result<problem_service_client::ProblemServiceClient<tonic::transport::Channel>> {
	Ok(problem_service_client::ProblemServiceClient::connect(crate::SERVER_URL).await?)
}
