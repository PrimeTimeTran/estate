use estate::prelude::*;

/// ## HID Daemon
/// - watch FS changes and write update to disk
/// - capture mouse events
/// - capture keyboard events
/// - stty -echo; trap 'stty echo' EXIT INT TERM; cargo run --bin daemon --features=native,daemon,macos
#[cfg(feature = "native")]
fn main() -> Result<()> {
	let host = Host::init().expect("Host should start successfully.");
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}
