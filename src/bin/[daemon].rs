use estate::prelude::*;

/// ## [Native Entry]
///
#[cfg(feature = "native")]
fn main() -> Result<()> {
	let host = Host::init().expect("Host should start successfully.");
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}
