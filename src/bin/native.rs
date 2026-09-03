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

use estate::app::{app_native::*, *};

#[cfg(feature = "web")]
fn wasm_init_flow() -> Result<()> {
	use estate::app::{App, app_web::WebApp};
	let mut app = App::<WebApp>::new()?;
	app.run(parsed)?;
	Ok(())
}

#[cfg(feature = "native")]
fn main() -> Result<()> {
	use cli;
	use estate::logger::*;
	let parsed = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&parsed)?;
	logger::init_logging(&config)?;
	let mut app = App::<NativeApp>::new()?;
	app.run(parsed)?;
	Ok(())
}
