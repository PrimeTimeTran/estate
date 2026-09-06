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

use estate::app::*;

#[cfg(feature = "native")]
fn main() -> Result<()> {
	use cli;
	use estate::app::*;
	let parsed = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&parsed)?;
	logger::init_logging(&config)?;
	let mut app = App::new();
	app.run(parsed)?;
	app.start()?;
	estate::app::tokio_main();

	Ok(())
}
// #[cfg(feature = "native")]
// #[tokio::main]
// fn main() -> Result<()> {
// 	use cli;
// 	use estate::app::*;
// 	let parsed = cli::context::parse();
// 	let mut config = LogConfig::load()?;
// 	config.apply_cli(&parsed)?;
// 	logger::init_logging(&config)?;
// 	let mut app = App::new();
// 	app.run(parsed)?;
// 	app.start()?;
// 	estate::app::test_main().await;
// 	Ok(())
// }
