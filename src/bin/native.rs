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

// lib.rs must have the following for logger import here to use.
// pub mod tool;
// pub use tool::*;

use estate::app::*;

#[cfg(feature = "native")]
fn main() -> Result<()> {
	use cli;
	use estate::app::*;
	let parsed = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&parsed)?;
	logger::init_logging(&config)?;
	let app = App::new();
	app.run(parsed)?;

	estate::app::test_main();
	Ok(())
}
