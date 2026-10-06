use crate::prelude::{
	keymap::{Action, Binding, Context},
	shared::Binding as OldKeyBinding,
	*,
};

impl SwiftNativeEvent {
	pub fn into_native_event(self) -> Option<NativeEvent> {
		let kind = match self.kind.as_str() {
			"key_down" => NativeEventKind::KeyDown {
				key_code: self.key_code?,
			},
			"key_up" => NativeEventKind::KeyUp {
				key_code: self.key_code?,
			},
			"flags_changed" => NativeEventKind::FlagsChanged {
				key_code: self.key_code?,
			},

			"frontmost_app" => {
				let app = self.frontmost_app.as_ref()?;
				NativeEventKind::FrontmostApp {
					name: app.name.clone(),
					bundle_id: app.bundle_id.clone(),
					pid: app.pid,
				}
			}

			"mouse_down" => NativeEventKind::MouseDown {
				button: self.button?,
				x: self.x?,
				y: self.y?,
			},

			"mouse_up" => NativeEventKind::MouseUp {
				button: self.button?,
				x: self.x?,
				y: self.y?,
			},

			"scroll" => NativeEventKind::Scroll {
				vertical: self.vertical?,
				horizontal: self.horizontal?,
			},

			_ => return None,
		};

		let modifiers = self.modifiers.unwrap_or_default();
		let scroll_x = Some(self.horizontal.unwrap_or(0) as f64);
		let scroll_y = Some(self.vertical.unwrap_or(0) as f64);

		Some(NativeEvent {
			scroll_x,
			scroll_y,
			sent_at: self.sent_at.or(self.timestamp).unwrap_or_default(),
			frontmost_app: self.frontmost_app.map(|app| FrontmostApp {
				name: app.name,
				bundle_id: app.bundle_id,
				pid: app.pid,
			}),
			kind,
			source: Some(self.source),
			timestamp: self.timestamp,
			key_code: self.key_code,
			name: self.name,
			direction: self
				.direction
				.and_then(|direction| match direction.as_str() {
					"down" => Some(keymap::KeyDirection::Down),
					"up" => Some(keymap::KeyDirection::Up),
					_ => None,
				}),

			modifiers: ModifierSnapshot {
				shift_left: modifiers.shift_left,
				shift_right: modifiers.shift_right,

				ctrl_left: modifiers.ctrl_left,
				ctrl_right: modifiers.ctrl_right,

				opt_left: modifiers.opt_left,
				opt_right: modifiers.opt_right,

				cmd_left: modifiers.cmd_left,
				cmd_right: modifiers.cmd_right,

				caps: modifiers.caps,
				function: modifiers.function,
			},
		})
	}
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct SwiftModifiers {
	#[serde(rename = "shift_left", default)]
	shift_left: bool,

	#[serde(rename = "shift_right", default)]
	shift_right: bool,

	#[serde(rename = "ctrl_left", default)]
	ctrl_left: bool,

	#[serde(rename = "ctrl_right", default)]
	ctrl_right: bool,

	#[serde(rename = "opt_left", default)]
	opt_left: bool,

	#[serde(rename = "opt_right", default)]
	opt_right: bool,

	#[serde(rename = "cmd_left", default)]
	cmd_left: bool,

	#[serde(rename = "cmd_right", default)]
	cmd_right: bool,

	#[serde(rename = "caps", default)]
	caps: bool,

	#[serde(rename = "function", default)]
	function: bool,
}
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SwiftCgEvent {
	#[serde(rename = "keyCode")]
	pub key_code: u16,

	#[serde(rename = "type")]
	pub event_type: u32,

	#[serde(rename = "flags")]
	pub flags: u64,

	#[serde(rename = "sessionFlags")]
	pub session_flags: u64,

	#[serde(rename = "sourcePID")]
	pub source_pid: i32,

	#[serde(rename = "sourceUserData")]
	pub source_user_data: u64,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SwiftNativeEvent {
	pub source: String,
	pub kind: String,

	#[serde(default)]
	pub direction: Option<String>,

	#[serde(default)]
	pub name: Option<String>,

	#[serde(default)]
	pub timestamp: Option<u64>,

	#[serde(default)]
	pub sent_at: Option<u64>,

	#[serde(default)]
	pub key_code: Option<u16>,

	#[serde(default)]
	pub button: Option<i64>,

	#[serde(default)]
	pub x: Option<f64>,

	#[serde(default)]
	pub y: Option<f64>,

	#[serde(default)]
	pub vertical: Option<i64>,

	#[serde(default)]
	pub horizontal: Option<i64>,

	#[serde(default)]
	pub modifiers: Option<SwiftModifiers>,

	#[serde(rename = "frontmostApp", default)]
	pub frontmost_app: Option<SwiftFrontmostApp>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SwiftFrontmostApp {
	#[serde(rename = "bundleID")]
	bundle_id: String,
	name: String,
	pid: i64,
}
#[derive(Debug, Deserialize)]
pub struct SwiftRawKeyEvent {
	kind: String,

	#[serde(default)]
	timestamp: u64,

	#[serde(default)]
	modifiers: Option<SwiftRawModifiers>,

	#[serde(default)]
	event: Option<SwiftRawCgEvent>,
}
#[derive(Debug, Deserialize)]
pub struct SwiftRawCgEvent {
	#[serde(rename = "keyCode")]
	key_code: u16,
}
#[derive(Debug, Deserialize, Default)]
pub struct SwiftRawModifiers {
	#[serde(rename = "leftShift", default)]
	shift_left: bool,

	#[serde(rename = "rightShift", default)]
	shift_right: bool,

	#[serde(rename = "leftCtrl", default)]
	ctrl_left: bool,

	#[serde(rename = "rightCtrl", default)]
	ctrl_right: bool,

	#[serde(rename = "leftOpt", default)]
	opt_left: bool,

	#[serde(rename = "rightOpt", default)]
	opt_right: bool,

	#[serde(rename = "leftCmd", default)]
	cmd_left: bool,

	#[serde(rename = "rightCmd", default)]
	cmd_right: bool,

	#[serde(rename = "caps", default)]
	caps: bool,

	#[serde(rename = "fn", default)]
	function: bool,
}
