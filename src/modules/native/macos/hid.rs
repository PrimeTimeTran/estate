use crate::prelude::{
	keymap::{Action, Binding, Context},
	shared::Binding as OldKeyBinding,
	*,
};
use anyhow::{Context as CtxAnyhow, Result};
use mach2::mach_time;
use std::process::{Child, Command, Stdio};

fn mach_now() -> u64 {
	unsafe { mach_time::mach_absolute_time() }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HidMessage {
	#[serde(rename = "ping")]
	Ping { id: u64 },

	#[serde(rename = "pong")]
	Pong { id: u64 },

	#[serde(rename = "native_event")]
	NativeEvent { event: NativeEvent },

	#[serde(rename = "action")]
	Action { action: String },
}

impl MacosHid {
	pub fn new() -> Result<Self> {
		tracing::info!("MacosHid new");

		let enabled = true;

		Ok(Self {
			socket: PathBuf::from("/tmp/estate-hid.sock"),
			child: None,

			bindings: vec![
				// Chord: Cmd + P
				Binding {
					trigger: Trigger::Chord {
						keys: vec![Key::MetaLeft, Key::Key("p".into())],
					},
					action: Action {
						name: "OpenCommandPalette".into(),
						description: "Open command palette".into(),
						code: "open_command_palette".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
				// Tap: Caps Lock
				Binding {
					trigger: Trigger::Tap { key: Key::CapsLock },
					action: Action {
						name: "OpenEstate".into(),
						description: "Open Estate".into(),
						code: "open_estate".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
				// Repeat: Shift twice
				Binding {
					trigger: Trigger::Repeat {
						key: Key::ShiftLeft,
						count: 2,
						max_interval_ms: Some(300),
					},
					action: Action {
						name: "QuickSwitcher".into(),
						description: "Open quick switcher".into(),
						code: "quick_switcher".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
				// Sequence: K then S
				Binding {
					trigger: Trigger::Sequence {
						steps: vec![
							Trigger::Tap {
								key: Key::Key("k".into()),
							},
							Trigger::Tap {
								key: Key::Key("s".into()),
							},
						],
						timeout_ms: Some(1000),
					},
					action: Action {
						name: "SaveAll".into(),
						description: "Save all".into(),
						code: "save_all".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
				// HoldThen: hold Shift, then tap P
				Binding {
					trigger: Trigger::HoldThen {
						held: Key::ShiftLeft,
						then: Box::new(Trigger::Tap {
							key: Key::Key("p".into()),
						}),
					},
					action: Action {
						name: "ShiftPalette".into(),
						description: "Open palette while holding Shift".into(),
						code: "shift_palette".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
				// WhileHeld: Shift + scroll up
				Binding {
					trigger: Trigger::WhileHeld {
						key: Key::ShiftLeft,
						trigger: Box::new(Trigger::Pointer {
							trigger: PointerTrigger::Scroll {
								axis: ScrollAxis::Vertical,
								direction: Some(keymap::ScrollDirection::Positive),
							},
						}),
					},
					action: Action {
						name: "NavigateUp".into(),
						description: "Navigate up while holding Shift".into(),
						code: "navigate_up".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
				// Ordered: Shift↓ A↓ A↑ Shift↑
				Binding {
					trigger: Trigger::Ordered {
						events: vec![
							GestureEvent::Down(Key::ShiftLeft),
							GestureEvent::Down(Key::Key("a".into())),
							GestureEvent::Up(Key::Key("a".into())),
							GestureEvent::Up(Key::ShiftLeft),
						],
					},
					action: Action {
						name: "OrderedTest".into(),
						description: "Test ordered gesture".into(),
						code: "ordered_test".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
				// Pointer: double click
				Binding {
					trigger: Trigger::Pointer {
						trigger: PointerTrigger::DoubleClick {
							button: MouseButton::Primary,
						},
					},
					action: Action {
						name: "Open".into(),
						description: "Open on double click".into(),
						code: "open".into(),
					},
					when: Context::default(),
					consume: Consume::Always,
					enabled,
				},
			],

			pressed: HashSet::new(),
			sequence: Vec::new(),
			last_tap: HashMap::new(),
		})
	}
	pub fn start(&mut self) -> Result<()> {
		todo!("strt ")
	}
	pub fn stop(&mut self) -> Result<()> {
		if let Some(mut child) = self.child.take() {
			tracing::info!(pid = child.id(), "🍎 stopping macOS HID");
			child.kill().ok();
			child.wait().ok();
		}
		Ok(())
	}
	pub async fn run(mut self, events: EventBus, cancel: CancellationToken) -> Result<()> {
		tracing::info!(
				socket = %self.socket.display(),
				"connecting to Swift HID"
		);

		let stream = UnixStream::connect(&self.socket).await.with_context(|| {
			format!(
				"failed to connect to Swift HID socket {}",
				self.socket.display()
			)
		})?;

		tracing::info!("connected to Swift HID");

		let (reader, mut writer) = stream.into_split();
		let mut reader = BufReader::new(reader);

		let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(10));

		let mut ping_id = 0u64;
		let mut line = String::new();

		loop {
			tokio::select! {
							_ = cancel.cancelled() => {
								tracing::info!(
									"macOS HID cancelled"
								);

								return Ok(());
							}
							_ = ping_interval.tick() => {
								ping_id += 1;

								let ping = HidMessage::Ping {
									id: ping_id,
								};

								let json =
									serde_json::to_string(&ping)?;

								writer
									.write_all(json.as_bytes())
									.await?;

								writer
									.write_all(b"\n")
									.await?;

								tracing::info!(
									id = ping_id,
									"🍏 Rust → Swift: PING"
								);
							}
							result = reader.read_line(&mut line) => {
							tracing::info!("🔥 RUST READ WAKE UP");
							let received_at = mach_now();
							let bytes = result?;
							tracing::info!(
								bytes,
								"🔥 RUST READ COMPLETE"
							);
							if bytes == 0 {
								tracing::warn!("Swift HID disconnected");
								return Ok(());
							}

							let raw = line.trim_end();

							tracing::info!(
								raw,
								"🍎 Swift → Rust"
							);
							let message =
								match serde_json::from_str::<HidMessage>(
									raw
								) {
									Ok(message) => message,

									Err(error) => {
										tracing::error!(
											%error,
											raw,
											"invalid HID message from Swift"
										);

										line.clear();
										continue;
									}
								};

							match message {
								HidMessage::Ping { id } => {
									tracing::info!(
										id,
										"🍎 Swift → Rust: PING"
									);

									let pong =
										HidMessage::Pong { id };

									let json =
										serde_json::to_string(&pong)?;

									writer
										.write_all(json.as_bytes())
										.await?;

									writer
										.write_all(b"\n")
										.await?;

									tracing::info!(
										id,
										"🍏 Rust → Swift: PONG"
									);
								}
								HidMessage::Pong { id } => {
									tracing::info!(
										id,
										"🍎 Swift → Rust: PONG"
									);
								}
								HidMessage::NativeEvent { event } => {
									let latency = received_at.saturating_sub(event.sent_at);

									tracing::info!(
										latency,
										"🔥 RUST EVENT"
									);

									for action in self.observe_event(&event) {
										tracing::info!(
											action = %action.name,
											code = %action.code,
											"🔥 ACTION"
										);

										// events.emi (/* action event */);
									}

									events.emit(event.into());
								}
			// 					HidMessage::NativeEvent { event } => {
			// 						let latency =
			// 							received_at
			// 								.saturating_sub(event.sent_at);
			//
			// 						tracing::info!(
			// 							latency,
			// 							"🔥 RUST EVENT"
			// 						);
			//
			// 						events.emit(event.into());
			// 					}
								HidMessage::Action { action } => {
									tracing::info!(
										%action,
										"🍎 Swift action received"
									);
								}
							}
							line.clear();
							}
						}
		}
	}
	pub fn observe_event(&mut self, event: &NativeEvent) -> Vec<Action> {
		let mut triggered = Vec::new();

		for index in 0..self.bindings.len() {
			if !self.bindings[index].enabled {
				continue;
			}

			let matched = {
				let trigger = &self.bindings[index].trigger.clone();
				self.matches(trigger, event)
			};

			if matched {
				let action = self.bindings[index].action.clone();

				tracing::info!(
					action = %action.name,
					code = %action.code,
					"🔥 HOTKEY TRIGGERED"
				);

				triggered.push(action);
			}
		}

		triggered
	}
	// 	fn matches(&mut self, trigger: &Trigger, event: &NativeEvent) -> bool {
	// 		match trigger {
	// 			Trigger::Chord { keys } => self.matches_chord(keys, event),
	//
	// 			Trigger::Tap { key } => self.matches_tap(key, event),
	//
	// 			Trigger::Repeat {
	// 				key,
	// 				count,
	// 				max_interval_ms,
	// 			} => self.matches_repeat(key, *count, *max_interval_ms, event),
	//
	// 			Trigger::Sequence { steps, timeout_ms } => self.matches_sequence(steps, *timeout_ms, event),
	//
	// 			Trigger::HoldThen { held, then } => self.matches_hold_then(held, then, event),
	//
	// 			Trigger::WhileHeld { key, trigger } => self.matches_while_held(key, trigger, event),
	//
	// 			Trigger::Ordered { events } => self.matches_ordered(events, event),
	//
	// 			Trigger::Pointer { trigger } => self.matches_pointer(trigger, event),
	// 		}
	// 	}
}

impl MacosHid {
	fn key_from_code(code: u16) -> Option<Key> {
		match code {
			// A-Z
			0x00 => Some(Key::Key("a".into())),
			0x01 => Some(Key::Key("s".into())),
			0x02 => Some(Key::Key("d".into())),
			0x03 => Some(Key::Key("f".into())),
			0x04 => Some(Key::Key("h".into())),
			0x05 => Some(Key::Key("g".into())),
			0x06 => Some(Key::Key("z".into())),
			0x07 => Some(Key::Key("x".into())),
			0x08 => Some(Key::Key("c".into())),
			0x09 => Some(Key::Key("v".into())),
			0x0B => Some(Key::Key("b".into())),
			0x0C => Some(Key::Key("q".into())),
			0x0D => Some(Key::Key("w".into())),
			0x0E => Some(Key::Key("e".into())),
			0x0F => Some(Key::Key("r".into())),
			0x10 => Some(Key::Key("y".into())),
			0x11 => Some(Key::Key("t".into())),
			0x12 => Some(Key::Key("1".into())),
			0x13 => Some(Key::Key("2".into())),
			0x14 => Some(Key::Key("3".into())),
			0x15 => Some(Key::Key("4".into())),
			0x16 => Some(Key::Key("6".into())),
			0x17 => Some(Key::Key("5".into())),
			0x18 => Some(Key::Key("=".into())),
			0x19 => Some(Key::Key("9".into())),
			0x1A => Some(Key::Key("7".into())),
			0x1B => Some(Key::Key("-".into())),
			0x1C => Some(Key::Key("8".into())),
			0x1D => Some(Key::Key("0".into())),
			0x1E => Some(Key::Key("]".into())),
			0x1F => Some(Key::Key("o".into())),
			0x20 => Some(Key::Key("u".into())),
			0x21 => Some(Key::Key("[".into())),
			0x22 => Some(Key::Key("i".into())),
			0x23 => Some(Key::Key("p".into())),
			0x24 => Some(Key::Enter),
			0x25 => Some(Key::Key("l".into())),
			0x26 => Some(Key::Key("j".into())),
			0x27 => Some(Key::Key("'".into())),
			0x28 => Some(Key::Key("k".into())),
			0x29 => Some(Key::Key(";".into())),
			0x2A => Some(Key::Key("\\".into())),
			0x2B => Some(Key::Key(",".into())),
			0x2C => Some(Key::Key("/".into())),
			0x2D => Some(Key::Key("n".into())),
			0x2E => Some(Key::Key("m".into())),
			0x2F => Some(Key::Key(".".into())),

			// Modifiers
			0x37 => Some(Key::MetaLeft),
			0x36 => Some(Key::MetaRight),

			0x38 => Some(Key::ShiftLeft),
			0x3C => Some(Key::ShiftRight),

			0x3A => Some(Key::AltLeft),
			0x3D => Some(Key::AltRight),

			0x3B => Some(Key::ControlLeft),
			0x3E => Some(Key::ControlRight),

			// Caps Lock
			0x39 => Some(Key::CapsLock),

			// Space
			0x31 => Some(Key::Space),

			// Tab
			0x30 => Some(Key::Tab),

			// Escape
			0x35 => Some(Key::Escape),

			// Delete
			0x33 => Some(Key::Backspace),

			// Arrow keys
			0x7E => Some(Key::ArrowUp),
			0x7D => Some(Key::ArrowDown),
			0x7B => Some(Key::ArrowLeft),
			0x7C => Some(Key::ArrowRight),

			_ => None,
		}
	}

	fn mouse_code(button: MouseButton) -> i64 {
		match button {
			MouseButton::Primary => 0,
			MouseButton::Secondary => 1,
			MouseButton::Middle => 2,
			MouseButton::Button4 => 3,
			MouseButton::Button5 => 4,
		}
	}

	fn mouse_button_matches(event: &NativeEvent, button: MouseButton) -> bool {
		let code = Self::mouse_code(button);

		matches!(
			event.kind,
			NativeEventKind::MouseUp {
				button: event_button,
				..
			} if event_button == code
		)
	}

	fn gesture_mouse_button(button: i64) -> Option<MouseButton> {
		match button {
			0 => Some(MouseButton::Primary),
			1 => Some(MouseButton::Secondary),
			2 => Some(MouseButton::Middle),
			3 => Some(MouseButton::Button4),
			4 => Some(MouseButton::Button5),
			_ => None,
		}
	}
	// fn mouse_code(button: MouseButton) -> i64 {
	// 	match button {
	// 		MouseButton::Primary => 0,
	// 		MouseButton::Secondary => 1,
	// 		MouseButton::Middle => 2,
	// 		MouseButton::Button4 => 3,
	// 		MouseButton::Button5 => 4,
	// 	}
	// }

	// fn mouse_button_matches(event: &NativeEvent, button: MouseButton) -> bool {
	// 	let code = Self::mouse_code(button);
	//
	// 	matches!(
	// 		event.kind,
	// 		NativeEventKind::MouseUp {
	// 			button: event_button,
	// 			..
	// 		} if event_button == code
	// 	)
	// }
	//
	// fn gesture_mouse_button(button: i64) -> Option<MouseButton> {
	// 	match button {
	// 		0 => Some(MouseButton::Primary),
	// 		1 => Some(MouseButton::Secondary),
	// 		2 => Some(MouseButton::Middle),
	// 		3 => Some(MouseButton::Button4),
	// 		4 => Some(MouseButton::Button5),
	// 		_ => None,
	// 	}
	// }

	fn matches(&mut self, trigger: &Trigger, event: &NativeEvent) -> bool {
		match trigger {
			Trigger::Chord { keys } => self.matches_chord(keys, event),

			Trigger::Tap { key } => self.matches_tap(key, event),

			Trigger::Repeat {
				key,
				count,
				max_interval_ms,
			} => self.matches_repeat(key, *count, *max_interval_ms, event),

			Trigger::Sequence { steps, timeout_ms } => self.matches_sequence(steps, *timeout_ms, event),

			Trigger::HoldThen { held, then } => self.matches_hold_then(held, then, event),

			Trigger::WhileHeld { key, trigger } => self.matches_while_held(key, trigger, event),

			Trigger::Ordered { events } => self.matches_ordered(events, event),

			Trigger::Pointer { trigger } => self.matches_pointer(trigger, event),
		}
	}
	fn matches_chord(&mut self, keys: &[Key], event: &NativeEvent) -> bool {
		self.update_pressed(event);

		let Some(key) = Self::event_key(event) else {
			return false;
		};

		matches!(event.kind, NativeEventKind::KeyDown { .. })
			&& keys.contains(&key)
			&& keys.iter().all(|key| self.pressed.contains(key))
	}

	fn matches_tap(&mut self, key: &Key, event: &NativeEvent) -> bool {
		let NativeEventKind::KeyUp { key_code } = event.kind else {
			return false;
		};

		let Some(event_key) = Self::key_from_code(key_code) else {
			return false;
		};

		if &event_key != key {
			return false;
		}

		self.last_tap.insert(key.clone(), std::time::Instant::now());

		true
	}

	fn matches_repeat(
		&mut self,
		key: &Key,
		count: u8,
		max_interval_ms: Option<u64>,
		event: &NativeEvent,
	) -> bool {
		let NativeEventKind::KeyUp { key_code } = event.kind else {
			return false;
		};

		let Some(event_key) = Self::key_from_code(key_code) else {
			return false;
		};

		if &event_key != key {
			return false;
		}

		let now = std::time::Instant::now();

		let within_window = self
			.last_tap
			.get(key)
			.map(|last| {
				max_interval_ms
					.map(|ms| now.duration_since(*last) <= std::time::Duration::from_millis(ms))
					.unwrap_or(true)
			})
			.unwrap_or(true);

		if !within_window {
			self.sequence.clear();
		}

		self.last_tap.insert(key.clone(), now);

		self.sequence.push(Trigger::Tap { key: key.clone() });

		if self.sequence.len() >= count as usize {
			self.sequence.clear();
			return true;
		}

		false
	}

	fn matches_sequence(
		&mut self,
		steps: &[Trigger],
		timeout_ms: Option<u64>,
		event: &NativeEvent,
	) -> bool {
		if steps.is_empty() {
			return false;
		}

		/*
		 * `sequence` stores the triggers we've already matched.
		 *
		 * This is intentionally simple for now. Once you want arbitrary
		 * nested sequences, give this its own SequenceState instead.
		 */

		let index = self.sequence.len();

		if index >= steps.len() {
			self.sequence.clear();
			return false;
		}

		if self.matches(&steps[index], event) {
			self.sequence.push(steps[index].clone());

			if self.sequence.len() == steps.len() {
				self.sequence.clear();
				return true;
			}

			return true;
		}

		// Optional timeout bookkeeping using the last tap.
		if let Some(timeout_ms) = timeout_ms {
			let expired = self
				.last_tap
				.values()
				.any(|last| last.elapsed() > std::time::Duration::from_millis(timeout_ms));

			if expired {
				self.sequence.clear();
			}
		}

		false
	}

	fn matches_hold_then(&mut self, held: &Key, then: &Trigger, event: &NativeEvent) -> bool {
		self.update_pressed(event);

		if !self.pressed.contains(held) {
			return false;
		}

		self.matches(then, event)
	}

	fn matches_while_held(&mut self, key: &Key, trigger: &Trigger, event: &NativeEvent) -> bool {
		self.update_pressed(event);

		if !self.pressed.contains(key) {
			return false;
		}

		self.matches(trigger, event)
	}

	fn matches_ordered(&mut self, expected: &[GestureEvent], event: &NativeEvent) -> bool {
		if expected.is_empty() {
			return false;
		}

		let Some(actual) = Self::gesture_event(event) else {
			return false;
		};

		let index = self.sequence.len();

		if index >= expected.len() {
			self.sequence.clear();
			return false;
		}

		if Self::gesture_matches(&expected[index], &actual) {
			self.sequence.push(Trigger::Tap {
				key: match &actual {
					GestureEvent::Down(key) | GestureEvent::Up(key) | GestureEvent::Tap(key) => key.clone(),

					_ => return false,
				},
			});

			if self.sequence.len() == expected.len() {
				self.sequence.clear();
				return true;
			}
		} else {
			self.sequence.clear();
		}

		false
	}

	fn matches_pointer(&mut self, trigger: &PointerTrigger, event: &NativeEvent) -> bool {
		match trigger {
			PointerTrigger::Click { button } => Self::matches_mouse_up(event, *button),

			PointerTrigger::DoubleClick { button } => {
				if !Self::matches_mouse_up(event, *button) {
					return false;
				}

				let now = std::time::Instant::now();

				let key = Key::Key(format!("__mouse_{button:?}"));

				let double = self
					.last_tap
					.get(&key)
					.map(|last| now.duration_since(*last) <= std::time::Duration::from_millis(300))
					.unwrap_or(false);

				self.last_tap.insert(key, now);

				double
			}

			PointerTrigger::Hold { button } => Self::matches_mouse_down(event, *button),

			PointerTrigger::Scroll { axis, direction } => {
				let NativeEventKind::Scroll {
					vertical,
					horizontal,
				} = &event.kind
				else {
					return false;
				};

				let amount = match axis {
					ScrollAxis::Vertical => *vertical,
					ScrollAxis::Horizontal => *horizontal,
				};

				if amount == 0 {
					return false;
				}

				match direction {
					None => true,
					Some(keymap::ScrollDirection::Positive) => amount > 0,
					Some(keymap::ScrollDirection::Negative) => amount < 0,
				}
			}

			PointerTrigger::Drag { button } => {
				// Requires MouseMove in NativeEventKind to implement properly.
				// For now, treat a mouse-down as the beginning of a drag.
				Self::matches_mouse_down(event, *button)
			}

			PointerTrigger::Move => {
				// Add MouseMove to NativeEventKind when you want this.
				false
			}
		}
	}

	fn update_pressed(&mut self, event: &NativeEvent) {
		match &event.kind {
			NativeEventKind::KeyDown { key_code } => {
				if let Some(key) = Self::key_from_code(*key_code) {
					self.pressed.insert(key);
				}
			}
			NativeEventKind::KeyUp { key_code } => {
				if let Some(key) = Self::key_from_code(*key_code) {
					self.pressed.remove(&key);
				}
			}
			_ => {}
		}
	}

	// fn update_pressed(&mut self, event: &NativeEvent) {
	// 	match event.kind {
	// 		NativeEventKind::KeyDown { key_code } => {
	// 			if let Some(key) = Self::key_from_code(key_code) {
	// 				self.pressed.insert(key);
	// 			}
	// 		}
	//
	// 		NativeEventKind::KeyUp { key_code } => {
	// 			if let Some(key) = Self::key_from_code(key_code) {
	// 				self.pressed.remove(&key);
	// 			}
	// 		}
	//
	// 		_ => {}
	// 	}
	// }

	fn gesture_event(event: &NativeEvent) -> Option<GestureEvent> {
		match &event.kind {
			NativeEventKind::KeyDown { key_code } => {
				Self::key_from_code(*key_code).map(GestureEvent::Down)
			}

			NativeEventKind::KeyUp { key_code } => Self::key_from_code(*key_code).map(GestureEvent::Up),

			NativeEventKind::MouseDown { button, .. } => {
				Self::gesture_mouse_button(*button).map(GestureEvent::MouseDown)
			}

			NativeEventKind::MouseUp { button, .. } => {
				Self::gesture_mouse_button(*button).map(GestureEvent::MouseUp)
			}

			NativeEventKind::Scroll {
				vertical,
				horizontal,
			} => Some(GestureEvent::Scroll {
				vertical: *vertical,
				horizontal: *horizontal,
			}),
		}
	}

	fn gesture_matches(expected: &GestureEvent, actual: &GestureEvent) -> bool {
		match (expected, actual) {
			(GestureEvent::Down(a), GestureEvent::Down(b)) => a == b,
			(GestureEvent::Up(a), GestureEvent::Up(b)) => a == b,
			(GestureEvent::Tap(a), GestureEvent::Tap(b)) => a == b,

			(GestureEvent::MouseDown(a), GestureEvent::MouseDown(b)) => a == b,

			(GestureEvent::MouseUp(a), GestureEvent::MouseUp(b)) => a == b,

			(
				GestureEvent::Scroll {
					vertical: av,
					horizontal: ah,
				},
				GestureEvent::Scroll {
					vertical: bv,
					horizontal: bh,
				},
			) => av == bv && ah == bh,

			_ => false,
		}
	}

	fn event_key(event: &NativeEvent) -> Option<Key> {
		match event.kind {
			NativeEventKind::KeyDown { key_code } | NativeEventKind::KeyUp { key_code } => {
				Self::key_from_code(key_code)
			}

			_ => None,
		}
	}

	fn matches_mouse_up(event: &NativeEvent, button: MouseButton) -> bool {
		let NativeEventKind::MouseUp {
			button: event_button,
			..
		} = &event.kind
		else {
			return false;
		};

		Self::gesture_mouse_button(*event_button)
			.map(|event_button| event_button == button)
			.unwrap_or(false)
	}

	fn matches_mouse_down(event: &NativeEvent, button: MouseButton) -> bool {
		matches!(
			&event.kind,
			NativeEventKind::MouseDown {
				button: event_button,
				..
			} if *event_button == Self::mouse_code(button)
		)
	}

	// fn gesture_event(event: &NativeEvent) -> Option<GestureEvent> {
	// 	match event.kind {
	// 		NativeEventKind::KeyDown { key_code } => Self::key_from_code(key_code).map(GestureEvent::Down),
	//
	// 		NativeEventKind::KeyUp { key_code } => Self::key_from_code(key_code).map(GestureEvent::Up),
	//
	// 		NativeEventKind::MouseDown { button, .. } => {
	// 			Self::gesture_mouse_button(button).map(GestureEvent::MouseDown)
	// 		}
	//
	// 		NativeEventKind::MouseUp { button, .. } => {
	// 			Self::gesture_mouse_button(button).map(GestureEvent::MouseUp)
	// 		}
	//
	// 		NativeEventKind::Scroll {
	// 			vertical,
	// 			horizontal,
	// 		} => Some(GestureEvent::Scroll {
	// 			vertical,
	// 			horizontal,
	// 		}),
	// 	}
	// }
	//
	// fn gesture_key(event: &GestureEvent) -> Option<Key> {
	// 	match event {
	// 		GestureEvent::Down(key) | GestureEvent::Up(key) | GestureEvent::Tap(key) => Some(key.clone()),
	// 		_ => None,
	// 	}
	// }
	//
	// fn gesture_matches(expected: &GestureEvent, actual: &GestureEvent) -> bool {
	// 	match (expected, actual) {
	// 		(GestureEvent::Down(a), GestureEvent::Down(b)) => a == b,
	// 		(GestureEvent::Up(a), GestureEvent::Up(b)) => a == b,
	// 		(GestureEvent::Tap(a), GestureEvent::Tap(b)) => a == b,
	//
	// 		(GestureEvent::MouseDown(a), GestureEvent::MouseDown(b)) => a == b,
	//
	// 		(GestureEvent::MouseUp(a), GestureEvent::MouseUp(b)) => a == b,
	//
	// 		(
	// 			GestureEvent::Scroll {
	// 				vertical: av,
	// 				horizontal: ah,
	// 			},
	// 			GestureEvent::Scroll {
	// 				vertical: bv,
	// 				horizontal: bh,
	// 			},
	// 		) => av == bv && ah == bh,
	//
	// 		_ => false,
	// 	}
	// }
}

pub struct MacosHid {
	socket: PathBuf,
	child: Option<std::process::Child>,

	pub bindings: Vec<Binding>,
	pub pressed: HashSet<Key>,
	pub sequence: Vec<Trigger>,
	pub last_tap: HashMap<Key, std::time::Instant>,
}

fn default_bindings() -> Vec<Binding> {
	vec![
		// ─────────────────────────────────────────────
		// Chord
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Chord {
				keys: vec![Key::MetaLeft, Key::Key("p".into())],
			},
			action: Action {
				name: "OpenCommandPalette".into(),
				description: "Open the command palette".into(),
				code: "open.command_palette".into(),
			},
			when: Context::default(),
			consume: Consume::Always,
			enabled: true,
		},
		Binding {
			trigger: Trigger::Chord {
				keys: vec![Key::MetaLeft, Key::ShiftLeft, Key::Key("o".into())],
			},
			action: Action {
				name: "OpenOutline".into(),
				description: "Open the outline".into(),
				code: "open.outline".into(),
			},
			when: Context::default(),
			consume: Consume::Always,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// Tap
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Tap { key: Key::CapsLock },
			action: Action {
				name: "OpenEstate".into(),
				description: "Open Estate".into(),
				code: "estate.open".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// Repeat
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Repeat {
				key: Key::ShiftLeft,
				count: 2,
				max_interval_ms: Some(300),
			},
			action: Action {
				name: "QuickSwitcher".into(),
				description: "Open quick switcher".into(),
				code: "open.quick_switcher".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// Sequence
		// Cmd+K → Cmd+S
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Sequence {
				steps: vec![
					Trigger::Chord {
						keys: vec![Key::MetaLeft, Key::Key("k".into())],
					},
					Trigger::Chord {
						keys: vec![Key::MetaLeft, Key::Key("s".into())],
					},
				],
				timeout_ms: Some(1000),
			},
			action: Action {
				name: "SaveAll".into(),
				description: "Save all files".into(),
				code: "files.save_all".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// HoldThen
		// Hold Shift → press P
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::HoldThen {
				held: Key::ShiftLeft,
				then: Box::new(Trigger::Tap {
					key: Key::Key("p".into()),
				}),
			},
			action: Action {
				name: "HoldShiftP".into(),
				description: "Shift held then P".into(),
				code: "test.hold_shift_p".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// WhileHeld
		// Hold Shift + scroll
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::WhileHeld {
				key: Key::ShiftLeft,
				trigger: Box::new(Trigger::Pointer {
					trigger: PointerTrigger::Scroll {
						axis: ScrollAxis::Vertical,
						direction: Some(keymap::ScrollDirection::Positive),
					},
				}),
			},
			action: Action {
				name: "NavigateUp".into(),
				description: "Navigate upward while Shift is held".into(),
				code: "navigate.up".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// Ordered physical events
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Ordered {
				events: vec![
					GestureEvent::Down(Key::ShiftLeft),
					GestureEvent::Down(Key::Key("a".into())),
					GestureEvent::Up(Key::Key("a".into())),
					GestureEvent::Up(Key::ShiftLeft),
				],
			},
			action: Action {
				name: "OrderedTest".into(),
				description: "Test ordered physical lifecycle".into(),
				code: "test.ordered".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// Pointer
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Pointer {
				trigger: PointerTrigger::DoubleClick {
					button: MouseButton::Primary,
				},
			},
			action: Action {
				name: "Open".into(),
				description: "Double click".into(),
				code: "open".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
		Binding {
			trigger: Trigger::Pointer {
				trigger: PointerTrigger::Scroll {
					axis: ScrollAxis::Horizontal,
					direction: Some(keymap::ScrollDirection::Positive),
				},
			},
			action: Action {
				name: "NavigateForward".into(),
				description: "Navigate forward".into(),
				code: "navigate.forward".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
	]
}
