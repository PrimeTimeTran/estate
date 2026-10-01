use std::io;

use estate::prelude::*;

// ═════════════════════════════════════════════════════════════════════════════
// Main
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
fn main() -> io::Result<()> {
	let mut input = platform::create_hdi_monitor();
	let result = input.run();
	result
}

//
// #[derive(Debug, Clone, Copy)]
// enum MouseButton {
// 	Left,
// 	Right,
// 	Middle,
// 	X1,
// 	X2,
// 	Other(u32),
// }
//
// #[derive(Debug, Clone, Copy)]
// enum KeyState {
// 	Down,
// 	Up,
// }
//
// #[derive(Debug)]
// enum InputEvent {
// 	Key {
// 		vk: u32,
// 		state: KeyState,
// 	},
//
// 	MouseButton {
// 		button: MouseButton,
// 		state: KeyState,
// 	},
//
// 	MouseMove {
// 		x: i32,
// 		y: i32,
// 	},
//
// 	MouseWheel {
// 		delta: i32,
// 	},
// }
