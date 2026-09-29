use std::io;

use estate::prelude::*;

// #[cfg(all(feature = "windows", target_arch = "windows"))]
fn main() -> io::Result<()> {
	let mut input = platform::create_hdi_monitor();
	let result = input.run();
	result
}
