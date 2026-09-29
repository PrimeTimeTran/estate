use crate::prelude::*;
use std::io;

#[derive(Debug, Clone, Copy)]
enum MouseButton {
	Left,
	Right,
	Middle,
	X1,
	X2,
	Other(u32),
}

#[derive(Debug, Clone, Copy)]
enum KeyState {
	Down,
	Up,
}

#[derive(Debug)]
enum InputEvent {
	Key {
		vk: u32,
		state: KeyState,
	},

	MouseButton {
		button: MouseButton,
		state: KeyState,
	},

	MouseMove {
		x: i32,
		y: i32,
	},

	MouseWheel {
		delta: i32,
	},
}

use std::ptr::null_mut;

use windows_sys::Win32::{
	Foundation::{LPARAM, LRESULT, WPARAM},
	UI::WindowsAndMessaging::{
		CallNextHookEx, DispatchMessageW, GetMessageW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG,
		MSLLHOOKSTRUCT, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL,
		WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
		WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
		XBUTTON1, XBUTTON2,
	},
};

pub struct HDIWindows {
	keyboard_hook: HHOOK,
	mouse_hook: HHOOK,
}

impl HDIWindows {
	pub fn new() -> Self {
		println!("WINDOWS: new()");

		Self {
			keyboard_hook: std::ptr::null_mut(),
			mouse_hook: std::ptr::null_mut(),
		}
	}

	fn install_hooks(&mut self) -> io::Result<()> {
		println!("WINDOWS: installing keyboard hook...");

		unsafe {
			self.keyboard_hook =
				SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), std::ptr::null_mut(), 0)
		}

		if self.keyboard_hook.is_null() {
			return Err(io::Error::last_os_error());
		}

		println!("WINDOWS: keyboard hook installed: {:?}", self.keyboard_hook);

		println!("WINDOWS: installing mouse hook...");

		unsafe {
			self.mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), std::ptr::null_mut(), 0);
		}

		if self.mouse_hook.is_null() {
			let error = io::Error::last_os_error();

			unsafe {
				UnhookWindowsHookEx(self.keyboard_hook);
			}

			self.keyboard_hook = std::ptr::null_mut();

			return Err(error);
		}

		println!("WINDOWS: mouse hook installed: {:?}", self.mouse_hook);

		Ok(())
	}

	fn uninstall_hooks(&mut self) {
		unsafe {
			if !self.keyboard_hook.is_null() {
				UnhookWindowsHookEx(self.keyboard_hook);
				self.keyboard_hook = std::ptr::null_mut();
			}

			if !self.mouse_hook.is_null() {
				UnhookWindowsHookEx(self.mouse_hook);
				self.mouse_hook = std::ptr::null_mut();
			}
		}

		println!("WINDOWS: hooks uninstalled");
	}
}

impl HDIInput for HDIWindows {
	fn run(&mut self) -> io::Result<()> {
		println!("WINDOWS: starting");

		self.install_hooks()?;

		println!();
		println!("════════════════════════════════════════════════════════════");
		println!(" WINDOWS INPUT SMOKE TEST");
		println!("════════════════════════════════════════════════════════════");
		println!(" Move your mouse");
		println!(" Click left/right/middle");
		println!(" Click mouse buttons 4/5");
		println!(" Scroll");
		println!(" Press/release keys");
		println!(" Ctrl+C to terminate");
		println!("════════════════════════════════════════════════════════════");
		println!();

		unsafe {
			let mut msg = std::mem::zeroed::<MSG>();

			loop {
				let result = GetMessageW(&mut msg, null_mut(), 0, 0);

				if result == -1 {
					let error = io::Error::last_os_error();

					self.uninstall_hooks();

					return Err(error);
				}

				if result == 0 {
					println!("WINDOWS: WM_QUIT received");
					break;
				}

				TranslateMessage(&msg);
				DispatchMessageW(&msg);
			}
		}

		self.uninstall_hooks();

		Ok(())
	}
}

impl Drop for HDIWindows {
	fn drop(&mut self) {
		unsafe {
			if !self.keyboard_hook.is_null() {
				UnhookWindowsHookEx(self.keyboard_hook);
			}

			if !self.mouse_hook.is_null() {
				UnhookWindowsHookEx(self.mouse_hook);
			}
		}
	}
}

// ═══════════════════════════════════════════════════════════════════════════
// Keyboard hook
// ═══════════════════════════════════════════════════════════════════════════

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
	if code == HC_ACTION as i32 {
		let event = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };

		let state = match wparam as u32 {
			WM_KEYDOWN => KeyState::Down,
			WM_KEYUP => KeyState::Up,

			_ => {
				return unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) };
			}
		};

		let input = InputEvent::Key {
			vk: event.vkCode,
			state,
		};

		println!(
			"KEY          {:?}  vk=0x{:02X}  scan=0x{:02X}",
			state, event.vkCode, event.scanCode,
		);

		let _ = input;
	}

	unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

// ═══════════════════════════════════════════════════════════════════════════
// Mouse hook
// ═══════════════════════════════════════════════════════════════════════════

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
	if code == HC_ACTION as i32 {
		let event = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };

		match wparam as u32 {
			WM_LBUTTONDOWN => {
				print_mouse(MouseButton::Left, KeyState::Down);
			}

			WM_LBUTTONUP => {
				print_mouse(MouseButton::Left, KeyState::Up);
			}

			WM_RBUTTONDOWN => {
				print_mouse(MouseButton::Right, KeyState::Down);
			}

			WM_RBUTTONUP => {
				print_mouse(MouseButton::Right, KeyState::Up);
			}

			WM_MBUTTONDOWN => {
				print_mouse(MouseButton::Middle, KeyState::Down);
			}

			WM_MBUTTONUP => {
				print_mouse(MouseButton::Middle, KeyState::Up);
			}

			WM_XBUTTONDOWN => {
				let button = xbutton(event.mouseData);

				print_mouse(button, KeyState::Down);
			}

			WM_XBUTTONUP => {
				let button = xbutton(event.mouseData);

				print_mouse(button, KeyState::Up);
			}

			WM_MOUSEMOVE => {
				// println!("MOUSE MOVE   x={} y={}", event.pt.x, event.pt.y,);

				let _ = InputEvent::MouseMove {
					x: event.pt.x,
					y: event.pt.y,
				};
			}

			WM_MOUSEWHEEL => {
				let delta = ((event.mouseData >> 16) & 0xffff) as i16;

				println!("MOUSE WHEEL  delta={}", delta,);

				let _ = InputEvent::MouseWheel {
					delta: delta as i32,
				};
			}

			_ => {}
		}
	}

	unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

fn xbutton(mouse_data: u32) -> MouseButton {
	let button = ((mouse_data >> 16) & 0xFFFF) as u16;

	match button {
		XBUTTON1 => MouseButton::X1,
		XBUTTON2 => MouseButton::X2,
		other => MouseButton::Other(other as u32),
	}
}

fn print_mouse(button: MouseButton, state: KeyState) {
	println!("MOUSE BUTTON {:?} {:?}", button, state,);

	let _ = InputEvent::MouseButton { button, state };
}

pub fn create_hdi_monitor() -> Box<dyn HDIInput> {
	println!("PLATFORM: Windows create()");

	let adapter = HDIWindows::new();

	println!("PLATFORM: Windows adapter constructed");

	Box::new(adapter)
}
