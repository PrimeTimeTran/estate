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
	let host = Host::init().expect("Host should start successfully.");
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}

// #[cfg(feature = "native")]
// fn _tokio_main() -> Result<()> {
// 	let host = Host::init().expect("Host should be there");
// 	let _parsed = host.run()?;
// 	let mut app = App::new(host)?;
// 	app.run()?;
// 	// estate::app::tokio_main().await;
// 	Ok(())
// }
