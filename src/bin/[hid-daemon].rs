use estate::prelude::*;

/// ## HID Daemon
/// - watch FS changes and write update to disk
/// - capture mouse events
/// - capture keyboard events
#[cfg(feature = "native")]
fn main() -> Result<()> {
	let host = Host::init().expect("Host should start successfully.");
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}
