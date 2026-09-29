use crate::prelude::*;

use std::{
	ffi::OsString,
	io,
	os::windows::ffi::OsStringExt,
	ptr::null_mut,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	thread::{self, JoinHandle},
};

use windows_sys::Win32::{
	Foundation::{HWINEVENTHOOK, HWND, LPARAM, LRESULT, WPARAM},
	UI::{
		Accessibility::{
			EVENT_SYSTEM_FOREGROUND, SetWinEventHook, UnhookWinEvent, WINEVENT_OUTOFCONTEXT,
		},
		WindowsAndMessaging::{
			DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextLengthW, GetWindowTextW,
			GetWindowThreadProcessId, MSG, TranslateMessage,
		},
	},
};

/// Platform-neutral description of the currently focused/foreground window.
#[derive(Debug, Clone)]
pub struct FocusWindow {
	pub hwnd: HWND,
	pub process_id: u32,
	pub title: String,
}

/// Receives notifications whenever the foreground window changes.
pub trait Focus {
	fn start(&mut self) -> io::Result<()>;
	fn stop(&mut self);
}

/// Windows implementation using WinEvent foreground notifications.
pub struct FocusWindows {
	running: Arc<AtomicBool>,
	thread: Option<JoinHandle<()>>,
}

impl FocusWindows {
	pub fn new() -> Self {
		Self {
			running: Arc::new(AtomicBool::new(false)),
			thread: None,
		}
	}

	fn spawn(&mut self) {
		let running = Arc::clone(&self.running);

		self.thread = Some(thread::spawn(move || {
			unsafe {
				let hook = SetWinEventHook(
					EVENT_SYSTEM_FOREGROUND,
					EVENT_SYSTEM_FOREGROUND,
					0,
					Some(foreground_event),
					0,
					0,
					WINEVENT_OUTOFCONTEXT,
				);

				if hook == 0 {
					eprintln!("FocusWindows: SetWinEventHook failed");
					return;
				}

				// Get the current foreground window immediately.
				//
				// This means consumers don't have to wait for the first
				// focus change before knowing the initial state.
				if let Some(window) = current_foreground_window() {
					print_focus(&window);
				}

				let mut msg = std::mem::zeroed::<MSG>();

				while running.load(Ordering::Acquire) {
					let result = GetMessageW(&mut msg, null_mut(), 0, 0);

					if result <= 0 {
						break;
					}

					TranslateMessage(&msg);
					DispatchMessageW(&msg);
				}

				UnhookWinEvent(hook);
			}
		}));
	}
}

impl Focus for FocusWindows {
	fn start(&mut self) -> io::Result<()> {
		if self.running.swap(true, Ordering::AcqRel) {
			return Ok(());
		}

		self.spawn();

		Ok(())
	}

	fn stop(&mut self) {
		if !self.running.swap(false, Ordering::AcqRel) {
			return;
		}

		/*
		 * The WinEvent callback thread is sitting inside GetMessageW().
		 *
		 * For a first implementation this is enough for normal operation,
		 * but eventually we should give the thread a proper shutdown
		 * message/event so stop() can wake it immediately.
		 */
	}
}

/// Called by Windows whenever the foreground window changes.
unsafe extern "system" fn foreground_event(
	_hook: HWINEVENTHOOK,
	event: u32,
	hwnd: HWND,
	_object_id: i32,
	_child_id: i32,
	_event_thread: u32,
	_event_time: u32,
) {
	if event != EVENT_SYSTEM_FOREGROUND {
		return;
	}

	if hwnd == 0 {
		return;
	}

	if let Some(window) = window_info(hwnd) {
		print_focus(&window);
	}
}

unsafe fn current_foreground_window() -> Option<FocusWindow> {
	let hwnd = GetForegroundWindow();

	if hwnd == 0 {
		return None;
	}

	window_info(hwnd)
}

unsafe fn window_info(hwnd: HWND) -> Option<FocusWindow> {
	let mut process_id = 0u32;

	GetWindowThreadProcessId(hwnd, &mut process_id);

	let title = window_title(hwnd);

	Some(FocusWindow {
		hwnd,
		process_id,
		title,
	})
}

unsafe fn window_title(hwnd: HWND) -> String {
	let length = GetWindowTextLengthW(hwnd);

	if length <= 0 {
		return String::new();
	}

	let mut buffer = vec![0u16; (length + 1) as usize];

	let written = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);

	OsString::from_wide(&buffer[..written as usize])
		.to_string_lossy()
		.into_owned()
}

fn print_focus(window: &FocusWindow) {
	println!(
		"[FOCUS] hwnd={:?} pid={} title={:?}",
		window.hwnd, window.process_id, window.title,
	);
}

pub fn create_focus_monitor() -> Box<dyn Focus> {
	Box::new(FocusWindows::new())
}
