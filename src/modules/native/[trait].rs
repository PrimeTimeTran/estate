use std::io;

use crate::prelude::*;

// Human Device Interaction Input
// - Keyboard
// - Mouse
// - Trackpad
pub trait HDIInput {
	fn run(&mut self) -> io::Result<()>;
}

pub trait Focus {
	fn start(&mut self) -> io::Result<()>;
	fn stop(&mut self);
}
