use core_foundation::{date::CFTimeInterval, runloop::kCFRunLoopDefaultMode};

use crate::{native::{CursorDaemon, CursorPosition}, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
	None,
	Up,
	Down,
	Left,
	Right,
}

pub fn scroll_state() -> &'static Mutex<ScrollRedirectState> {
	SCROLL_STATE.get_or_init(|| {
		Mutex::new(ScrollRedirectState {
			active: false,
			redirected: false,
			original_position: CGPoint { x: 0.0, y: 0.0 },
			target_position: CGPoint { x: 0.0, y: 0.0 },
		})
	})
}

pub fn target_position(bounds: CGRect, target: ScreenPosition, y: f64) -> CGPoint {
	let inset = CURSOR_INSET;
	let inset = inset.clamp(0.0, 0.5);
	let x = match target {
		ScreenPosition::Left => bounds.origin.x + bounds.size.width * inset,
		ScreenPosition::Right => bounds.origin.x + bounds.size.width * (1.0 - inset),
		ScreenPosition::Center => bounds.origin.x + bounds.size.width * 0.5,
	};
	CGPoint { x, y }
}

pub static SCROLL_STATE: OnceLock<Mutex<ScrollRedirectState>> = OnceLock::new();

#[derive(Debug)]
pub struct GestureState {
	pub active_focus: FocusedPane,
	pub side_panel_width: f32,
	pub secondary_scroll_offset: f32,
	pub last_delta: egui::Vec2,
	pub last_direction: ScrollDirection,
}
impl Default for GestureState {
	fn default() -> Self {
		Self {
			active_focus: FocusedPane::MainEditor,
			side_panel_width: 300.0,
			secondary_scroll_offset: 0.0,
			last_delta: egui::Vec2::ZERO,
			last_direction: ScrollDirection::None,
		}
	}
}
#[derive(Debug)]
pub struct GestureController {
	pub state: GestureState,
}
impl GestureController {
	pub(crate) fn new() -> Self {
		Self {
			state: GestureState::default(),
		}
	}
	pub(crate) fn inspect(&mut self, ui: &egui::Ui, input: &IOState) -> TrackpadState {
		let current = ui.input(|input| {
			let delta = input.smooth_scroll_delta;
			(
				delta,
				input.modifiers.shift,
				input.modifiers.ctrl,
				input.modifiers.alt,
				input.modifiers.command,
				input.pointer.hover_pos(),
			)
		});
		let (delta, shift, ctrl, alt, command, mouse_pos) = current;
		if delta != egui::Vec2::ZERO {
			self.state.last_delta = delta;
			self.state.last_direction = Self::direction(delta);
		}
		TrackpadState {
			alt_held: alt,
			clicked: None,
			command_held: input.command_held,
			ctrl_held: ctrl,
			delta: self.state.last_delta,
			direction: self.state.last_direction,
			focus: self.state.active_focus,
			hovered: input.cursor_target,
			mouse_pos: input.cursor_pos,
			shift_held: shift,
		}
	}
	pub(crate) fn direction(delta: egui::Vec2) -> ScrollDirection {
		if delta.x == 0.0 && delta.y == 0.0 {
			ScrollDirection::None
		} else if delta.x.abs() > delta.y.abs() {
			if delta.x > 0.0 {
				ScrollDirection::Right
			} else {
				ScrollDirection::Left
			}
		} else if delta.y > 0.0 {
			ScrollDirection::Down
		} else {
			ScrollDirection::Up
		}
	}
	pub(crate) fn hover_target(
		mouse_pos: Option<egui::Pos2>,
		viewport: egui::Rect,
	) -> Option<CursorTarget> {
		let Some(pos) = mouse_pos else {
			return None;
		};
		if !viewport.contains(pos) {
			return None;
		}
		Some(CursorTarget::Main)
	}
	pub(crate) fn focus_for_target(target: CursorTarget) -> FocusedPane {
		match target {
			CursorTarget::Main => FocusedPane::MainEditor,
			CursorTarget::DockLeft | CursorTarget::DockRight => FocusedPane::SidePanel,
			CursorTarget::BottomPanel
			| CursorTarget::ActivityBar
			| CursorTarget::PrimaryBar
			| CursorTarget::SecondaryBar
			| CursorTarget::StatusBar
			| CursorTarget::None => FocusedPane::Unknown,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollRedirectState {
	pub active: bool,
	pub redirected: bool,
	pub original_position: CGPoint,
	pub target_position: CGPoint,
}
#[derive(Debug, Clone, Copy)]
pub struct TrackpadState {
	pub delta: egui::Vec2,
	pub direction: ScrollDirection,
	pub shift_held: bool,
	pub ctrl_held: bool,
	pub alt_held: bool,
	pub command_held: bool,
	pub mouse_pos: Option<egui::Pos2>,
	pub hovered: CursorTarget,
	pub clicked: Option<CursorTarget>,
	pub focus: FocusedPane,
}
impl TrackpadState {
	pub(crate) fn primary_axis(&self) -> &'static str {
		if self.delta.x.abs() > self.delta.y.abs() {
			"Horizontal (X)"
		} else if self.delta.y.abs() > self.delta.x.abs() {
			"Vertical (Y)"
		} else {
			"None"
		}
	}
	pub(crate) fn hovered_name(&self) -> &'static str {
		self.hovered.name()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum CursorEvent {
	CursorPosition { x: f64, y: f64 },
	ModifiersChanged(Modifiers),
}

pub struct AppCursorSink {
	pub tx: std::sync::mpsc::Sender<CursorEvent>,
}

impl CursorEventSink for AppCursorSink {
	fn cursor_moved(&self, position: CursorPosition) {
		let _ = self.tx.send(CursorEvent::CursorPosition {
			x: position.x,
			y: position.y,
		});
	}

	fn modifiers_changed(&self, modifiers: Modifiers) {
		let _ = self.tx.send(CursorEvent::ModifiersChanged(modifiers));
	}
}

impl<S> CursorDaemon<S>
where
	S: CursorEventSink,
{
	pub fn new(sink: S, cancel: CancellationToken) -> Self {
		Self { sink, cancel }
	}
	pub fn run(self) -> anyhow::Result<()> {
		let trusted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();

		if !trusted {
			tracing::warn!("Accessibility permission not granted; cursor daemon stopped");
			return Ok(());
		}
		let cancel = self.cancel.clone();
		let callback = {
			move |_proxy_cg: CGEventTapProxy,
			      event_type: CGEventType,
			      event: &CGEvent|
			      -> CallbackResult { self.handle_event(event_type, event) }
		};

		let tap = match CGEventTap::new(
			CGEventTapLocation::HID,
			CGEventTapPlacement::HeadInsertEventTap,
			CGEventTapOptions::Default,
			vec![
				CGEventType::ScrollWheel,
				CGEventType::FlagsChanged,
				CGEventType::MouseMoved,
				CGEventType::KeyDown,
			],
			callback,
		) {
			Ok(tap) => tap,
			Err(error) => {
				tracing::error!("Failed to create CGEventTap: {:?}", error);
				return Ok(());
			}
		};

		// unsafe {
		// 	let port = tap.mach_port();
		// 	let source = match port.create_runloop_source(0) {
		// 		Ok(source) => source,
		// 		Err(_) => {
		// 			tracing::error!("Failed to create CFRunLoopSource");
		// 			return Ok(());
		// 		}
		// 	};
		// 	tap.enable();

		// 	while !cancel.is_cancelled() {
		// 		CFRunLoop::run_in_mode(
		// 			kCFRunLoopDefaultMode,
		// 			std::time::Duration::from_millis(100),
		// 			false,
		// 		);
		// 	}
		// }
		unsafe {
			let port = tap.mach_port();

			let source = match port.create_runloop_source(0) {
				Ok(source) => source,
				Err(_) => {
					tracing::error!("Failed to create CFRunLoopSource");
					return Ok(());
				}
			};

			let run_loop = CFRunLoop::get_current();

			run_loop.add_source(&source, kCFRunLoopCommonModes);

			tap.enable();

			while !cancel.is_cancelled() {
				CFRunLoop::run_in_mode(
					kCFRunLoopDefaultMode,
					std::time::Duration::from_millis(100),
					false,
				);
			}
		}
		Ok(())
	}

	fn handle_event(&self, event_type: CGEventType, event: &CGEvent) -> CallbackResult {
		match event_type {
			CGEventType::MouseMoved => self.mouse_moved(event),
			CGEventType::FlagsChanged => self.flags_changed(event),
			CGEventType::ScrollWheel => self.scroll_wheel(event),
			CGEventType::KeyDown => self.key_down(event),
			_ => CallbackResult::Keep,
		}
	}

	fn mouse_moved(&self, event: &CGEvent) -> CallbackResult {
		if REDIRECTING_SCROLL.load(Ordering::Relaxed) {
			return CallbackResult::Keep;
		}

		let p = event.location();
		let position = CursorPosition { x: p.x, y: p.y };
		self.sink.cursor_moved(position);

		CallbackResult::Keep
	}

	fn flags_changed(&self, event: &CGEvent) -> CallbackResult {
		let flags = event.get_flags();

		let shift = flags.contains(CGEventFlags::CGEventFlagShift);
		let ctrl = flags.contains(CGEventFlags::CGEventFlagControl);
		let alt = flags.contains(CGEventFlags::CGEventFlagAlternate);
		let command = flags.contains(CGEventFlags::CGEventFlagCommand);

		// Get the previous state BEFORE updating it.
		let was_shift_down = SHIFT_HELD.swap(shift, Ordering::Relaxed);

		// -------------------------------------------------------------
		// Tell the application immediately about modifier state.
		// -------------------------------------------------------------

		let mut modifiers = Modifiers::empty();

		if shift {
			modifiers.insert(Modifiers::SHIFT);
		}

		if ctrl {
			modifiers.insert(Modifiers::CONTROL);
		}

		if alt {
			modifiers.insert(Modifiers::ALT);
		}

		if command {
			modifiers.insert(Modifiers::META);
		}

		// Tell the application immediately about modifier state.
		self.sink.modifiers_changed(modifiers);

		// -------------------------------------------------------------
		// SHIFT DOWN
		// -------------------------------------------------------------

		if shift && !was_shift_down {
			let location = event.location();

			let bounds = CGDisplay::main().bounds();

			let midpoint = bounds.origin.x + bounds.size.width * 0.5;

			let target = if location.x < midpoint {
				ScreenPosition::Right
			} else {
				ScreenPosition::Left
			};

			let target_position = target_position(bounds, target, location.y);

			let mut state = scroll_state().lock().unwrap();

			state.active = true;
			state.redirected = true;
			state.original_position = location;
			state.target_position = target_position;

			if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
				if let Ok(move_event) = CGEvent::new_mouse_event(
					source,
					CGEventType::MouseMoved,
					target_position,
					CGMouseButton::Left,
				) {
					move_event.post(CGEventTapLocation::HID);
				}
			}
		}

		// -------------------------------------------------------------
		// SHIFT UP
		// -------------------------------------------------------------

		if !shift && was_shift_down {
			let mut state = scroll_state().lock().unwrap();

			let original = state.original_position;

			if state.active {
				if let Ok(source) = CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
					if let Ok(restore_event) = CGEvent::new_mouse_event(
						source,
						CGEventType::MouseMoved,
						original,
						CGMouseButton::Left,
					) {
						restore_event.post(CGEventTapLocation::HID);
					}
				}
			}

			state.active = false;
			state.redirected = false;
		}

		CallbackResult::Keep
	}

	fn scroll_wheel(&self, _event: &CGEvent) -> CallbackResult {
		if !SHIFT_HELD.load(Ordering::Relaxed) {
			return CallbackResult::Keep;
		}

		let state = scroll_state().lock().unwrap();

		if !state.active {
			return CallbackResult::Keep;
		}

		// Keep your existing scroll redirection logic here.
		//

		// The important part is that the daemon owns the
		// CGEvent processing, while the sink only receives
		// application-level events.

		CallbackResult::Keep
	}

	fn key_down(&self, event: &CGEvent) -> CallbackResult {
		let keycode =
			event.get_integer_value_field(core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE);

		match keycode {
			// 18 => {
			//     move_cursor_to(ScreenPosition::Left);
			// }
			// 19 => {
			//     move_cursor_to(ScreenPosition::Center);
			// }
			// 20 => {
			//     move_cursor_to(ScreenPosition::Right);
			// }
			_ => {}
		}

		CallbackResult::Keep
	}
}

pub fn spawn_global_cursor_daemon_new<S>(sink: S, cancel: CancellationToken) -> anyhow::Result<()>
where
	S: CursorEventSink,
{
	CursorDaemon::new(sink, cancel).run()
}
