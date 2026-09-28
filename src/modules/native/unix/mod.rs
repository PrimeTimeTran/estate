use crate::prelude::*;
use std::io;

pub mod server;
pub use server::*;

pub mod client;
pub use client::*;

pub fn create_hdi_monitor() -> Box<dyn HDIInput> {
	todo!("")
}

// ═════════════════════════════════════════════════════════════════════════════
// Unsupported platforms
// ═════════════════════════════════════════════════════════════════════════════

struct UnsupportedInputAdapter;

impl HDIInput for UnsupportedInputAdapter {
	fn run(&mut self) -> io::Result<()> {
		Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"input smoke test is not implemented for this platform",
		))
	}
}

pub fn create() -> Box<dyn HDIInput> {
	println!("PLATFORM: unsupported platform");

	Box::new(UnsupportedInputAdapter)
}
