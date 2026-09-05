use crate::native::prelude::*;

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
pub fn spawn_global_cursor_daemon(proxy: EventLoopProxy<AppEvent>) {
	std::thread::spawn(move || {
		// May need to grant permissions multiple times if the user runs the app from a different tool?
		// When I run from Zed/VSCode it runs fine. But Ghosty it doesn't The scroll tracker doesn't activate
		let trusted = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
		if !trusted {
			return;
		}
		let callback = move |_proxy_cg: CGEventTapProxy,
		                     event_type: CGEventType,
		                     event: &CGEvent|
		      -> CallbackResult {
			match event_type {
				CGEventType::MouseMoved => {
					if REDIRECTING_SCROLL.load(Ordering::Relaxed) {
						return CallbackResult::Keep;
					}
					let location = event.location();
					let _ = proxy.send_event(AppEvent::CursorPosition {
						x: location.x,
						y: location.y,
					});
					CallbackResult::Keep
				}
				CGEventType::FlagsChanged => {
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
					let _ = proxy.send_event(AppEvent::ModifiersChanged {
						shift,
						ctrl,
						alt,
						command,
					});
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
				CGEventType::KeyDown => {
					let keycode =
						event.get_integer_value_field(core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE);
					match keycode {
						// 18 => {
						// 	println!("Key '1' pressed");
						// 	move_cursor_to(ScreenPosition::Left);
						// }
						// 19 => {
						// 	println!("Key '2' pressed");
						// 	move_cursor_to(ScreenPosition::Center);
						// }
						// 20 => {
						// 	println!("Key '3' pressed");
						// 	move_cursor_to(ScreenPosition::Right);
						// }
						_ => {}
					}
					CallbackResult::Keep
				}
				CGEventType::ScrollWheel => {
					if !SHIFT_HELD.load(Ordering::Relaxed) {
						return CallbackResult::Keep;
					}

					let state = scroll_state().lock().unwrap();

					if !state.active {
						return CallbackResult::Keep;
					}
					CallbackResult::Keep
				}
				_ => CallbackResult::Keep,
			}
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
				eprintln!("❌ Failed to create CGEventTap: {:?}", error);
				return;
			}
		};
		unsafe {
			let port = tap.mach_port();
			let source = match port.create_runloop_source(0) {
				Ok(source) => source,
				Err(_) => {
					eprintln!("❌ Failed to create CFRunLoopSource");
					return;
				}
			};
			let run_loop = CFRunLoop::get_current();
			run_loop.add_source(&source, kCFRunLoopCommonModes);
			tap.enable();
			CFRunLoop::run_current();
		}
	});
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
	None,
	Up,
	Down,
	Left,
	Right,
}
