use crate::prelude::*;

pub mod server;
pub use server::*;

pub mod client;
pub use client::*;

pub struct HDIUnix;

impl HDIUnix {
	pub fn new() -> Self {
		println!("Unix: new()");
		Self
	}
}

impl Default for HDIUnix {
	fn default() -> Self {
		Self::new()
	}
}

impl HDIInput for HDIUnix {
	fn run(&mut self) -> std::io::Result<()> {
		Err(std::io::Error::new(
			std::io::ErrorKind::Unsupported,
			"Unix input monitoring backend is not implemented yet",
		))
	}
}

pub fn create_hdi_monitor() -> Box<dyn HDIInput> {
	println!("PLATFORM: Unix create()");
	Box::new(HDIUnix::new())
}
