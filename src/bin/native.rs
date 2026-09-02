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

// #[cfg(feature = "native")]
fn main() -> Result<()> {
	let cli = Cli::parse();
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	{
		let mut config = LogConfig::load()?;
		config.apply_cli(&cli)?;
		logger::init_logging(&config)?;

		return Ok(Self {
			native: NativeApp::new()?,
		});
	}
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;

	let app = App::new()?;

	NativeApp::new()?.run(app, cli)
}

// fn main() -> Result<()> {
// 	App::new(cli::context::parse())?.run()
// }
