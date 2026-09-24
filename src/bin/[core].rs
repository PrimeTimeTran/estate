use estate::prelude::*;

/// ## [Native Entry]
///
#[cfg(feature = "native,daemon")]
fn main() -> Result<()> {
	// $ cargo run --bin core --features=native,daemon
	let host = Host::init().expect("Host should start successfully.");
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}
