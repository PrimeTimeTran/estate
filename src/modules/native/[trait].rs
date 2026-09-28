use std::io;

use crate::prelude::*;

// Human Device Interaction Input
// - Keyboard
// - Mouse
// - Trackpad
pub trait HDIInput {
	fn run(&mut self) -> io::Result<()>;
}
