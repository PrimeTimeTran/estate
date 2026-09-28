// Human Device Interaction Input
// - Keyboard
// - Mouse
// - Trackpad
trait HDIInput {
	fn run(&mut self) -> io::Result<()>;
}
