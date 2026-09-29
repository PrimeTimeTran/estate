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

use estate::prelude::*;

/// ## [Native Entry]
///
// #[cfg(feature = "native")]
fn main() -> Result<()> {
  let host = Host::init().unwrap_or_else(|err| {
      eprintln!("Host::init() failed:\n{err:#}");
      std::process::exit(1);
  });
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}
