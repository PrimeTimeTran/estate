use crate::prelude::{
	keymap::{Action, Binding, Context},
	shared::Binding as OldKeyBinding,
	*,
};

use anyhow::{Context as CtxAnyhow, Result};
use mach2::mach_time;
use std::process::{Child, Command, Stdio};

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

fn mach_now() -> u64 {
	unsafe { mach_time::mach_absolute_time() }
}
fn parse_message(line: &str) -> anyhow::Result<HidMessage> {
	let value: serde_json::Value = serde_json::from_str(line)?;
	Ok(serde_json::from_value(value)?)
}
fn key_state(active: bool, symbol: &str) -> &str {
	if active { symbol } else { "·" }
}
fn display_name(value: &str, max: usize) -> String {
	let mut chars = value.chars();

	let truncated: String = chars.by_ref().take(max).collect();

	if chars.next().is_some() {
		format!("{truncated}…")
	} else {
		truncated
	}
}
fn default_bindings() -> Vec<Binding> {
	vec![
		// ─────────────────────────────────────────────
		// Chord: Caps Lock + P
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Chord {
				keys: vec![Key::CapsLock, Key::Key("p".into())],
			},
			action: Action {
				name: "TestCapsP".into(),
				description: "Test Caps Lock + P".into(),
				code: "test.caps_p".into(),
			},
			when: Context::default(),
			consume: Consume::Always,
			enabled: true,
		},
		// ─────────────────────────────────────────────
		// Double tap: Caps Lock
		// ─────────────────────────────────────────────
		Binding {
			trigger: Trigger::Repeat {
				key: Key::CapsLock,
				count: 2,
				max_interval_ms: Some(300),
			},
			action: Action {
				name: "TestCapsDoubleTap".into(),
				description: "Test double tap Caps Lock".into(),
				code: "test.caps_double_tap".into(),
			},
			when: Context::default(),
			consume: Consume::OnMatch,
			enabled: true,
		},
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
fn key_from_code(code: u16) -> Option<Key> {
	match code {
		// ─────────────────────────────────────────────
		// Alphanumeric
		// ─────────────────────────────────────────────
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

		// Number row
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

		// Letters / punctuation
		0x1E => Some(Key::Key("]".into())),
		0x1F => Some(Key::Key("o".into())),
		0x20 => Some(Key::Key("u".into())),
		0x21 => Some(Key::Key("[".into())),
		0x22 => Some(Key::Key("i".into())),
		0x23 => Some(Key::Key("p".into())),
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

		0x32 => Some(Key::Key("`".into())),
		0x34 => Some(Key::Key("]".into())),

		// ─────────────────────────────────────────────
		// Editing / whitespace
		// ─────────────────────────────────────────────
		0x24 => Some(Key::Enter),
		0x30 => Some(Key::Tab),
		0x31 => Some(Key::Space),
		0x33 => Some(Key::Backspace),
		0x35 => Some(Key::Escape),

		// ─────────────────────────────────────────────
		// Modifiers
		// ─────────────────────────────────────────────
		0x37 => Some(Key::MetaLeft),
		0x36 => Some(Key::MetaRight),

		0x38 => Some(Key::ShiftLeft),
		0x3C => Some(Key::ShiftRight),

		0x3A => Some(Key::AltLeft),
		0x3D => Some(Key::AltRight),

		0x3B => Some(Key::ControlLeft),
		0x3E => Some(Key::ControlRight),

		0x39 => Some(Key::CapsLock),

		// Fn / Globe
		0x3F => Some(Key::Function),

		// ─────────────────────────────────────────────
		// Function keys
		// ─────────────────────────────────────────────
		0x7A => Some(Key::F(1)),
		0x78 => Some(Key::F(2)),
		0x63 => Some(Key::F(3)),
		0x76 => Some(Key::F(4)),
		0x60 => Some(Key::F(5)),
		0x61 => Some(Key::F(6)),
		0x62 => Some(Key::F(7)),
		0x64 => Some(Key::F(8)),
		0x65 => Some(Key::F(9)),
		0x6D => Some(Key::F(10)),
		0x67 => Some(Key::F(11)),
		0x6F => Some(Key::F(12)),
		0x69 => Some(Key::F(13)),
		0x6B => Some(Key::F(14)),
		0x71 => Some(Key::F(15)),
		0x6A => Some(Key::F(16)),
		0x40 => Some(Key::F(17)),
		0x4F => Some(Key::F(18)),
		0x50 => Some(Key::F(19)),
		0x5A => Some(Key::F(20)),

		// ─────────────────────────────────────────────
		// Navigation
		// ─────────────────────────────────────────────
		0x7B => Some(Key::ArrowLeft),
		0x7C => Some(Key::ArrowRight),
		0x7D => Some(Key::ArrowDown),
		0x7E => Some(Key::ArrowUp),

		0x73 => Some(Key::Home),
		0x77 => Some(Key::End),
		0x74 => Some(Key::PageUp),
		0x79 => Some(Key::PageDown),

		// Forward Delete
		0x75 => Some(Key::Delete),

		// Help
		0x72 => Some(Key::Help),

		// ─────────────────────────────────────────────
		// Numeric keypad
		// ─────────────────────────────────────────────
		0x52 => Some(Key::Key("0".into())),
		0x53 => Some(Key::Key("1".into())),
		0x54 => Some(Key::Key("2".into())),
		0x55 => Some(Key::Key("3".into())),
		0x56 => Some(Key::Key("4".into())),
		0x57 => Some(Key::Key("5".into())),
		0x58 => Some(Key::Key("6".into())),
		0x59 => Some(Key::Key("7".into())),
		0x5B => Some(Key::Key("8".into())),
		0x5C => Some(Key::Key("9".into())),

		0x41 => Some(Key::Key(".".into())),
		0x43 => Some(Key::Key("*".into())),
		0x45 => Some(Key::Key("+".into())),
		0x4B => Some(Key::Key("/".into())),
		0x4E => Some(Key::Key("-".into())),
		0x51 => Some(Key::Key("=".into())),

		// Keypad Enter
		0x4C => Some(Key::Enter),

		// Keypad Clear / Num Lock
		0x47 => Some(Key::NumLock),
		0x5F => Some(Key::NumLock),

		// ─────────────────────────────────────────────
		// Keypad navigation
		// ─────────────────────────────────────────────
		0x54 => Some(Key::Key("2".into())),
		0x55 => Some(Key::Key("3".into())),
		0x56 => Some(Key::Key("4".into())),
		0x57 => Some(Key::Key("5".into())),
		0x58 => Some(Key::Key("6".into())),
		0x59 => Some(Key::Key("7".into())),
		0x5B => Some(Key::Key("8".into())),
		0x5C => Some(Key::Key("9".into())),

		// ─────────────────────────────────────────────
		// Unknown / unsupported
		// ─────────────────────────────────────────────
		_ => None,
	}
}

impl MacosHid {
	fn handle_event(&mut self, event: NativeEvent, received_at: u64, events: &EventBus) {
		self.state_update(&event);
		let latency = received_at.saturating_sub(event.sent_at);
		for action in self.observe_event(&event) {
			tracing::debug!(
					action = %action.name,
					code = %action.code,
					latency,
					"🔥 HOTKEY TRIGGERED"
			);
		}
		// events.emit(event.into());
		// Only publish native events that Estate cares about.
		// if let Some(event) = event.into_event() {
		// 	events.emit(event);
		// }
		if let Ok(event) = event.try_into() {
			events.emit(event);
		}
		// let latency = received_at.saturating_sub(event.sent_at);
		// self.log_native_event(&event);
		// for action in self.observe_event(&event) {
		// 	tracing::debug!(
		// 			action = %action.name,
		// 			code = %action.code,
		// 			"🔥 HOTKEY TRIGGERED"
		// 	);
		// }
		// events.emit(event.into());
	}
	fn state_update(&mut self, event: &NativeEvent) {
		tracing::info!("MacosHid state_update");
		let m = &event.modifiers;
		self.state.shift_left = m.shift_left;
		self.state.shift_right = m.shift_right;
		self.state.ctrl_left = m.ctrl_left;
		self.state.ctrl_right = m.ctrl_right;
		self.state.opt_left = m.opt_left;
		self.state.opt_right = m.opt_right;
		self.state.cmd_left = m.cmd_left;
		self.state.cmd_right = m.cmd_right;
		self.state.caps = m.caps;
		self.state.function = m.function;
		if let Some(app) = &event.frontmost_app {
			self.state.frontmost_app = app.name.clone();
		}
		self.print_event(event);
	}
	fn print_event(&mut self, event: &NativeEvent) {
		let s = &self.state;
		let app = display_name(&s.frontmost_app, 10);
		let left = format!(
			"{} {} {} {}",
			key_state(s.shift_left, "⇧"),
			key_state(s.ctrl_left, "⌃"),
			key_state(s.opt_left, "⌥"),
			key_state(s.cmd_left, "⌘"),
		);
		let right = format!(
			"{} {} {} {}",
			key_state(s.shift_right, "⇧"),
			key_state(s.ctrl_right, "⌃"),
			key_state(s.opt_right, "⌥"),
			key_state(s.cmd_right, "⌘"),
		);
		let special = format!("{} {}", key_state(s.function, "fn"), key_state(s.caps, "⇪"),);
		let event_display = match event.kind {
			NativeEventKind::Scroll {
				vertical,
				horizontal,
			} => {
				let x = event.scroll_x.unwrap_or(0.0);
				let y = event.scroll_y.unwrap_or(0.0);

				if y.abs() >= x.abs() {
					if y > 0.0 {
						format!("🖱 ↑ WHEEL {:>5.1}", y)
					} else if y < 0.0 {
						format!("🖱 ↓ WHEEL {:>5.1}", y.abs())
					} else {
						"🖱 · WHEEL".to_string()
					}
				} else if x > 0.0 {
					format!("🖱 → WHEEL {:>5.1}", x)
				} else if x < 0.0 {
					format!("🖱 ← WHEEL {:>5.1}", x.abs())
				} else {
					"🖱 · WHEEL".to_string()
				}
			}

			_ => {
				let arrow = match event.direction {
					Some(keymap::KeyDirection::Down) => "↓",
					Some(keymap::KeyDirection::Up) => "↑",
					None => " ",
				};

				let name = event.name.as_deref().unwrap_or("");
				format!("{arrow} {name}")
			}
		};
		let key_code = event
			.key_code
			.map(|code| code.to_string())
			.unwrap_or_default();
		println!(
			"{:<10} | {:<17} | {:<17} | {:<7} | {:<16} | {:>5}",
			app, left, right, special, event_display, key_code,
		);
	}
}
impl MacosHid {
	pub fn new() -> Result<Self> {
		Self::init_hid_smoke_log();
		let enabled = true;
		Ok(Self {
			state: HidState::default(),
			socket: PathBuf::from(HID_SOCKET),
			child: None,
			bindings: default_bindings(),
			pressed: HashSet::new(),
			sequence: Vec::new(),
			last_tap: HashMap::new(),
		})
	}
	pub fn start(&mut self) -> Result<()> {
		if self.child.is_some() {
			tracing::debug!("🍎 macOS OS observer already running");
			return Ok(());
		}

		// Remove any socket left behind by a previous observer.
		if self.socket.exists() {
			tracing::info!(
				socket = %self.socket.display(),
				"🍎 removing stale macOS HID socket"
			);

			std::fs::remove_file(&self.socket).with_context(|| {
				format!(
					"failed to remove stale macOS HID socket: {}",
					self.socket.display()
				)
			})?;
		}

		let source_dir = "/Users/future/kb/project/crates/estate/src/modules/native/macos/native";

		let source = format!("{source_dir}/os-observer.swift");
		let shim = format!("{source_dir}/hid-event-shim.o");
		let output = "/tmp/estate-os-observer";

		tracing::debug!("🍎 building macOS OS observer");

		let build = std::process::Command::new("swiftc")
			.current_dir(source_dir)
			.args([&source, &shim, "-o", output])
			.output()
			.context("failed to invoke swiftc")?;

		if !build.status.success() {
			anyhow::bail!(
				"failed to build macOS OS observer:\n{}{}",
				String::from_utf8_lossy(&build.stdout),
				String::from_utf8_lossy(&build.stderr),
			);
		}

		tracing::debug!("🍎 macOS OS observer built: {output}");

		let child = std::process::Command::new(output)
			.stdin(std::process::Stdio::null())
			.stdout(std::process::Stdio::inherit())
			.stderr(std::process::Stdio::inherit())
			.spawn()
			.context("failed to start /tmp/estate-os-observer")?;

		tracing::info!(
			pid = child.id(),
			socket = %self.socket.display(),
			"🍎 macOS OS observer started"
		);

		self.child = Some(child);

		Ok(())
	}
	pub fn stop(&mut self) -> Result<()> {
		if let Some(mut child) = self.child.take() {
			tracing::debug!(pid = child.id(), "🍎 stopping macOS HID");
			child.kill().ok();
			child.wait().ok();
		}
		Ok(())
	}
	pub async fn run(mut self, events: EventBus, cancel: CancellationToken) -> Result<()> {
		tracing::info!("MacHid runrunrunrunrun");
		let stream = self.connect().await?;

		tracing::info!("🔥 RUST GOT UNIX STREAM");

		let (reader, mut writer) = stream.into_split();

		tracing::info!("🔥 RUST ENTERING HID LOOP");
		let mut reader = BufReader::new(reader);
		let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(10));
		let mut ping_id = 0u64;
		let mut line = String::new();
		loop {
			tokio::select! {
							_ = cancel.cancelled() => {
									tracing::debug!("macOS HID cancelled");
									return Ok(());
							}
							_ = ping_interval.tick() => {
									ping_id += 1;
									self.send_ping(&mut writer, ping_id).await?;
							}
							result = reader.read_line(&mut line) => {
			tracing::info!("🔥 RUST READ COMPLETED");

			let received_at = mach_now();

			let bytes = match result {
					Ok(bytes) => {
							tracing::info!(bytes, "🔥 RUST READ RESULT");
							bytes
					}

					Err(error) => {
							tracing::error!(
									%error,
									"🔥 RUST READ ERROR"
							);
							return Err(error.into());
					}
			};

			if bytes == 0 {
					tracing::warn!(
							"🔥 RUST READ EOF — Swift HID disconnected"
					);
					return Ok(());
			}

			tracing::info!(
					line = %line.trim_end(),
					"🔥 RUST RECEIVED HID"
			);

			self.handle_line(
					line.trim_end(),
					received_at,
					&mut writer,
					&events,
			).await?;

			line.clear();
							}
					}
		}
	}
	async fn connect(&self) -> Result<UnixStream> {
		tracing::debug!(
				socket = %self.socket.display(),
				"🍎 connecting to macOS HID"
		);

		loop {
			match UnixStream::connect(&self.socket).await {
				Ok(stream) => {
					tracing::debug!(
							socket = %self.socket.display(),
							"🍎 connected to macOS HID"
					);

					return Ok(stream);
				}

				Err(error) => {
					tracing::debug!(
							%error,
							socket = %self.socket.display(),
							"waiting for macOS HID socket"
					);

					tokio::time::sleep(std::time::Duration::from_millis(100)).await;
				}
			}
		}
	}
	async fn send_ping(&self, writer: &mut tokio::net::unix::OwnedWriteHalf, id: u64) -> Result<()> {
		let message = HidMessage::Ping { id };
		self.send_message(writer, &message).await?;
		tracing::debug!(id, "🍏 Rust → Swift: PING");
		Ok(())
	}
	async fn send_message(
		&self,
		writer: &mut tokio::net::unix::OwnedWriteHalf,
		message: &HidMessage,
	) -> Result<()> {
		let json = serde_json::to_string(message)?;

		writer.write_all(json.as_bytes()).await?;
		writer.write_all(b"\n").await?;

		Ok(())
	}
	async fn handle_ping(
		&self,
		id: u64,
		writer: &mut tokio::net::unix::OwnedWriteHalf,
	) -> Result<()> {
		// tracing::debug!(id, "🍎 Swift → Rust: PING");
		let pong = HidMessage::Pong { id };
		self.send_message(writer, &pong).await?;
		tracing::debug!(id, "🍏 Rust → Swift: PONG");
		Ok(())
	}
	async fn handle_line(
		&mut self,
		raw: &str,
		received_at: u64,
		writer: &mut tokio::net::unix::OwnedWriteHalf,
		events: &EventBus,
	) -> Result<()> {
		let Some(message) = self.parse_message(raw) else {
			return Ok(());
		};
		match message {
			HidMessage::Ping { id } => {
				self.handle_ping(id, writer).await?;
			}
			HidMessage::Pong { id } => {
				self.handle_pong(id);
			}
			HidMessage::NativeEvent { event } => {
				self.handle_event(event, received_at, events);
			}
			HidMessage::Action { action } => {
				self.handle_action(action);
			}
		}
		Ok(())
	}
	fn parse_message(&self, raw: &str) -> Option<HidMessage> {
		let value: serde_json::Value = match serde_json::from_str(raw) {
			Ok(value) => value,
			Err(error) => {
				tracing::error!(
					%error,
					raw,
					"invalid JSON from Swift"
				);
				return None;
			}
		};
		if value.get("type").and_then(|v| v.as_str()) == Some("native_event") {
			let event_value = value.get("event")?;
			let swift_event: SwiftNativeEvent = match serde_json::from_value(event_value.clone()) {
				Ok(event) => event,
				Err(error) => {
					tracing::error!(
						%error,
						event = %event_value,
						"❌ Swift native_event does not match wire schema"
					);
					return None;
				}
			};
			let event = match swift_event.into_native_event() {
				Some(event) => event,
				None => {
					tracing::warn!(raw, "unsupported Swift native event");
					return None;
				}
			};
			return Some(HidMessage::NativeEvent { event });
		}
		match serde_json::from_value::<HidMessage>(value) {
			Ok(message) => Some(message),
			Err(error) => None,
		}
	}
	fn handle_action(&self, action: String) {
		// tracing::debug!(
		// 		action = %action,
		// 		"🍎 Swift action received"
		// );
	}
	fn log_native_event(&self, event: &NativeEvent) {
		let now = event.sent_at;
		let (left, right, special, event_display, flags_text, key_code) = match &event.kind {
			NativeEventKind::KeyDown { key_code } => {
				let key = Self::key_from_code(*key_code);
				let (left, right, special) = Self::key_side_display(key.as_ref(), &event.modifiers);
				(
					left,
					right,
					special,
					key
						.as_ref()
						.map(|key| key.display())
						.unwrap_or_else(|| "?".into()),
					"KEY↓".into(),
					*key_code as i64,
				)
			}
			NativeEventKind::KeyUp { key_code } => {
				let key = Self::key_from_code(*key_code);
				let (left, right, special) = Self::key_side_display(key.as_ref(), &event.modifiers);
				(
					left,
					right,
					special,
					key
						.as_ref()
						.map(|key| key.display())
						.unwrap_or_else(|| "?".into()),
					"KEY↑".into(),
					*key_code as i64,
				)
			}

			NativeEventKind::FlagsChanged { key_code } => {
				let key = Self::key_from_code(*key_code);
				let (left, right, special) = Self::key_side_display(key.as_ref(), &event.modifiers);
				let direction = if Self::modifier_is_down(key.as_ref(), &event.modifiers) {
					"↓"
				} else {
					"↑"
				};
				(
					left,
					right,
					special,
					key
						.as_ref()
						.map(|key| key.display())
						.unwrap_or_else(|| "?".into()),
					format!("MOD{direction}"),
					*key_code as i64,
				)
			}
			NativeEventKind::MouseDown { .. } => (
				"·".into(),
				"·".into(),
				"·".into(),
				"MOUSE".into(),
				"MOUSE↓".into(),
				-1,
			),
			NativeEventKind::MouseUp { .. } => (
				"·".into(),
				"·".into(),
				"·".into(),
				"MOUSE".into(),
				"MOUSE↑".into(),
				-1,
			),
			NativeEventKind::Scroll { .. } => (
				"·".into(),
				"·".into(),
				"·".into(),
				"SCROLL".into(),
				"SCROLL".into(),
				-1,
			),
			other => (
				"·".into(),
				"·".into(),
				"·".into(),
				format!("{other:?}"),
				"OTHER".into(),
				-1,
			),
		};
		let flags: u64 = 0;
		let session_flags: u64 = 0;
		let source_pid: i64 = 0;
		let source_user_data: i64 = 0;
		tracing::debug!(
			"{} | {:<9} | {:<9} | {:<4} | {:<14} | {:<8} | {:>4} | {:>10} | {:>10} | {:>6} | {:>6}",
			now,
			left,
			right,
			special,
			event_display,
			flags_text,
			key_code,
			flags,
			session_flags,
			source_pid,
			source_user_data,
		);
	}
	fn log_key_event(&self, event: &NativeEvent) {
		match &event.kind {
			NativeEventKind::KeyDown { key_code } => {
				let key = Self::key_from_code(*key_code);

				// tracing::debug!(
				// key_code,
				// key = ?key,
				// "🔥 KEY DOWN"
				// );
			}

			NativeEventKind::KeyUp { key_code } => {
				let key = Self::key_from_code(*key_code);
				//
				// tracing::debug!(
				// key_code,
				// key = ?key,
				// "🔥 KEY UP"
				// );
			}

			other => {
				tracing::debug!(
						kind = ?other,
						"🔥 OTHER NATIVE EVENT KIND"
				);
			}
		}
	}
	fn handle_pong(&self, id: u64) {
		tracing::debug!(id, "🍎 Swift → Rust: PONG");
	}
	fn modifier_display(&self, modifiers: &ModifierSnapshot) -> String {
		let mut parts = Vec::new();
		if modifiers.shift_left {
			parts.push("⇧L");
		}
		if modifiers.shift_right {
			parts.push("⇧R");
		}
		if modifiers.ctrl_left {
			parts.push("⌃L");
		}
		if modifiers.ctrl_right {
			parts.push("⌃R");
		}
		if modifiers.opt_left {
			parts.push("⌥L");
		}
		if modifiers.opt_right {
			parts.push("⌥R");
		}
		if modifiers.cmd_left {
			parts.push("⌘L");
		}
		if modifiers.cmd_right {
			parts.push("⌘R");
		}
		if modifiers.caps {
			parts.push("⇪");
		}
		if modifiers.function {
			parts.push("fn");
		}
		if parts.is_empty() {
			"·".to_string()
		} else {
			parts.join(" ")
		}
	}
	fn observe_modifier(&mut self, event: &NativeEvent) -> Vec<Action> {
		let NativeEventKind::FlagsChanged { key_code } = &event.kind else {
			return vec![];
		};

		let Some(key) = Self::key_from_code(*key_code) else {
			// tracing::warn!(key_code = *key_code, "unknown modifier key code");
			return vec![];
		};

		tracing::debug!(
			"MODIFIER {:?} code={} modifiers={}",
			key,
			key_code,
			self.modifier_display(&event.modifiers),
		);

		// Modifier events establish/update state.
		//
		// Do not perform shortcut matching here yet.
		// We first want to verify that the incoming modifier snapshot
		// is correct for every flagsChanged event.

		vec![]
	}
	fn observe_key(&mut self, event: &NativeEvent) -> Vec<Action> {
		// 		let NativeEventKind::KeyDown { key_code } = &event.kind else {
		// 			return vec![];
		// 		};
		//
		// 		let key_code = *key_code;
		//
		// 		let Some(key) = Self::key_from_code(key_code) else {
		// 			tracing::warn!(key_code, "KEY DOWN received but key code is not normalized");
		// 			return vec![];
		// 		};

		// tracing::debug!(
		// 	"🔑 KEY DOWN | key={:?} display={} code={} modifiers={}",
		// 	key,
		// 	key.display(),
		// 	key_code,
		// 	self.modifier_display(&event.modifiers),
		// );
		vec![]
	}
	pub fn observe_event(&mut self, event: &NativeEvent) -> Vec<Action> {
		match &event.kind {
			// NativeEventKind::KeyDown { .. } => self.observe_key(event),
			// Everything else intentionally disabled while normalizing KeyDown.
			_ => vec![],
		}
	}
	fn observe_mouse(&mut self, event: &NativeEvent) -> Vec<Action> {
		match &event.kind {
			NativeEventKind::MouseDown { button, .. } => {
				tracing::debug!(
					"MOUSE DOWN button={} modifiers={}",
					button,
					self.modifier_display(&event.modifiers),
				);
			}
			NativeEventKind::MouseUp { button, .. } => {
				tracing::debug!(
					"MOUSE UP button={} modifiers={}",
					button,
					self.modifier_display(&event.modifiers),
				);
			}
			_ => return vec![],
		}
		vec![]
	}
}
impl MacosHid {
	/// Convert an Estate action code string into a macOS keycode.
	pub fn keycode_from_string(code: &str) -> Option<u16> {
		code.parse::<u16>().ok()
	}
	pub fn key_from_code(code: u16) -> Option<Key> {
		key_from_code(code)
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

	fn gesture_event(event: &NativeEvent) -> Option<GestureEvent> {
		match &event.kind {
			NativeEventKind::KeyDown { key_code } => {
				Self::key_from_code(*key_code).map(GestureEvent::Down)
			}
			NativeEventKind::FlagsChanged { .. } => None,
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
			_ => None,
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

	fn write_hid_trace(event: &NativeEvent, action: Option<&Action>) -> std::io::Result<()> {
		let path = std::env::current_dir()?.join("estate-hid-smoke.log");
		let mut file = OpenOptions::new().create(true).append(true).open(path)?;
		let event_name = match &event.kind {
			NativeEventKind::KeyDown { .. } => "↓",
			NativeEventKind::KeyUp { .. } => "↑",
			NativeEventKind::MouseDown { .. } => "M↓",
			NativeEventKind::MouseUp { .. } => "M↑",
			NativeEventKind::Scroll { .. } => "SCROLL",
			NativeEventKind::FlagsChanged { .. } => "⇄",
			_ => "UNKNOWN",
		};
		writeln!(
			file,
			"{:<19} | {:<24} | {:<24}",
			format_timestamp(event.sent_at as f64),
			event_name,
			action
				.map(|a| format!("{} ({})", a.name, a.code))
				.unwrap_or_default(),
		)?;
		Ok(())
	}
	fn init_hid_smoke_log() -> std::io::Result<()> {
		let path = std::env::current_dir()?.join(ESTATE_HID_SMOKE_LOG);
		let mut file = std::fs::File::create(path)?;
		writeln!(
			file,
			"TIME          LS LC LO LM | RS RC RO RM | FN CP | EVENT              FLAGS"
		)?;
		writeln!(
			file,
			"              -- --------- | --------- | -- -- | ------------------ ----------------"
		)?;
		writeln!(
			file,
			"Columns: LS LC LO LM = left modifiers, RS RC RO RM = right modifiers, FN CP = Fn/Caps"
		)?;
		writeln!(
			file,
			"Event flags are aggregate CoreGraphics state; left/right state is reconstructed from keycodes"
		)?;
		writeln!(file)?;
		Ok(())
	}
	fn handle_modifier_event(&mut self, event: &SwiftNativeEvent) {
		if event.kind != "flags_changed" {
			return;
		}
		let Some(key_code) = event.key_code else {
			tracing::warn!(
				name = ?event.name,
				"🍎 MODIFIER EVENT missing key code"
			);
			return;
		};
		let key = MacosHid::key_from_code(key_code);
		tracing::debug!(
			name = ?event.name,
			key_code,
			key = ?key.map(|k| k.display()),
			direction = ?event.direction,
			modifiers = ?event.modifiers,
			"🍎 MODIFIER EVENT"
		);
	}
}
impl MacosHid {
	fn key_side_display(key: Option<&Key>, modifiers: &ModifierSnapshot) -> (String, String, String) {
		let left = match key {
			Some(Key::ShiftLeft) | Some(Key::ControlLeft) | Some(Key::AltLeft) | Some(Key::MetaLeft) => {
				key.unwrap().display()
			}
			_ => {
				let mut parts = Vec::new();
				if modifiers.shift_left {
					parts.push("⇧");
				}
				if modifiers.ctrl_left {
					parts.push("⌃");
				}
				if modifiers.opt_left {
					parts.push("⌥");
				}
				if modifiers.cmd_left {
					parts.push("⌘");
				}
				if parts.is_empty() {
					"·".into()
				} else {
					parts.join("")
				}
			}
		};
		let right = match key {
			Some(Key::ShiftRight)
			| Some(Key::ControlRight)
			| Some(Key::AltRight)
			| Some(Key::MetaRight) => key.unwrap().display(),
			_ => {
				let mut parts = Vec::new();
				if modifiers.shift_right {
					parts.push("⇧");
				}
				if modifiers.ctrl_right {
					parts.push("⌃");
				}
				if modifiers.opt_right {
					parts.push("⌥");
				}
				if modifiers.cmd_right {
					parts.push("⌘");
				}
				if parts.is_empty() {
					"·".into()
				} else {
					parts.join("")
				}
			}
		};
		let mut special_parts = Vec::new();
		if modifiers.caps {
			special_parts.push("⇪");
		}
		if modifiers.function {
			special_parts.push("fn");
		}
		let special = if special_parts.is_empty() {
			"·".into()
		} else {
			special_parts.join(" ")
		};
		(left, right, special)
	}
	fn modifier_is_down(key: Option<&Key>, modifiers: &ModifierSnapshot) -> bool {
		match key {
			Some(Key::ShiftLeft) => modifiers.shift_left,
			Some(Key::ShiftRight) => modifiers.shift_right,

			Some(Key::ControlLeft) => modifiers.ctrl_left,
			Some(Key::ControlRight) => modifiers.ctrl_right,

			Some(Key::AltLeft) => modifiers.opt_left,
			Some(Key::AltRight) => modifiers.opt_right,

			Some(Key::MetaLeft) => modifiers.cmd_left,
			Some(Key::MetaRight) => modifiers.cmd_right,

			Some(Key::CapsLock) => modifiers.caps,
			Some(Key::Function) => modifiers.function,
			_ => false,
		}
	}
}

impl MacosHid {
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
}
#[derive(Debug, Default, Clone)]
pub struct HidState {
	shift_left: bool,
	shift_right: bool,

	ctrl_left: bool,
	ctrl_right: bool,

	opt_left: bool,
	opt_right: bool,

	cmd_left: bool,
	cmd_right: bool,

	caps: bool,
	function: bool,
	frontmost_app: String,
}
pub struct HidObserver {
	state: HidState,
}
pub struct MacosHid {
	socket: PathBuf,
	child: Option<std::process::Child>,
	state: HidState,
	pub bindings: Vec<Binding>,
	pub pressed: HashSet<Key>,
	pub sequence: Vec<Trigger>,
	pub last_tap: HashMap<Key, std::time::Instant>,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifierSnapshot {
	pub shift_left: bool,
	pub shift_right: bool,

	pub ctrl_left: bool,
	pub ctrl_right: bool,

	pub opt_left: bool,
	pub opt_right: bool,

	pub cmd_left: bool,
	pub cmd_right: bool,

	pub caps: bool,
	pub function: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifiers {
	#[serde(rename = "leftShift")]
	pub shift_left: bool,

	#[serde(rename = "rightShift")]
	pub shift_right: bool,

	#[serde(rename = "leftCmd")]
	pub cmd_left: bool,

	#[serde(rename = "rightCmd")]
	pub cmd_right: bool,

	#[serde(rename = "leftOpt")]
	pub opt_left: bool,

	#[serde(rename = "rightOpt")]
	pub opt_right: bool,

	#[serde(rename = "leftCtrl")]
	pub ctrl_left: bool,

	#[serde(rename = "rightCtrl")]
	pub ctrl_right: bool,

	#[serde(rename = "fn")]
	pub fn_key: bool,

	pub caps: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontmostApp {
	pub pid: i64,

	#[serde(rename = "bundleID")]
	pub bundle_id: String,

	pub name: String,
}
