use std::io;

use crate::prelude::*;

// Human Device Int`eraction Input
// - Keyboard
// - Mouse
// - Trackpad
trait HDIInput {
	fn run(&mut self) -> io::Result<()>;
}
