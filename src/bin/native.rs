// Estate Engine CLI.

// Install:
// ```sh
// cargo install --path . --bin estate
// ```

// Usage:
// Start Estate Engine:
// ```sh
// estate start
// ```

// ```sh
// estate fmt path/to/file.rs
// ```

// lib.rs must have th efollowing for logger import here to use.
// pub mod tool;
// pub use tool::*;
use estate::{logger::*, prelude::*};

fn main2() -> Result<()> {
	// use estate::api::WasmApiClient;
	// let state = EstateState::default();
	// let runtime = WasmRuntime::new(state);
	// let engine = EstateEngine::new(runtime)?;
	// let api = Arc::new(WasmApiClient::new("http://localhost:3000"));
	// let _app = AppRuntime::new(engine, api);
	Ok(())
}

// #[cfg(feature = "native")]
fn main() -> Result<()> {
	use cli;
	let parsed = cli::context::parse();
	use estate::api::NativeApiClient;
	let state = EstateState::default();
	let mut config = LogConfig::load()?;
	config.apply_cli(&parsed)?;
	logger::init_logging(&config)?;
	let app = App::new()?;
	app.run(parsed);
	Ok(())
	// NativeRuntime::new(handle)
	// NativeApp::new()?.run(parsed)
}

// fn main() -> Result<()> {
// 	App::new(cli::context::parse())?.run()
// }
