use crate::{model::resolver::crate_root, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
	pub name: String,
	pub description: String,
	pub code: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionKeybindingDefault {
	pub name: String,
	pub description: String,
	pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
	pub trigger: Trigger,
	pub action: Action,

	#[serde(default)]
	pub when: Context,

	#[serde(default)]
	pub consume: Consume,

	#[serde(default)]
	pub enabled: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Consume {
	/// Always consume the input when this binding is active.
	#[default]
	Always,

	/// Never consume the input; let it continue to the OS/apps.
	Never,

	/// Consume the input only when the binding actually matches.
	OnMatch,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
	/// Application bundle ID.
	///
	/// dev.zed.Zed
	/// com.microsoft.VSCode
	/// com.jetbrains.rustrover
	#[serde(default)]
	pub apps: Vec<String>,

	/// Optional workspace/project.
	#[serde(default)]
	pub workspace: Option<String>,

	/// Optional mode.
	///
	/// editor / terminal / explorer / outline / etc.
	#[serde(default)]
	pub mode: Option<String>,

	/// Optional context predicates.
	#[serde(default)]
	pub predicates: Vec<Predicate>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Predicate {
	ForegroundApp {
		bundle_id: String,
	},

	Workspace {
		path: String,
	},

	Mode {
		name: String,
	},

	/// Useful later for things like:
	/// "only when an editor has focus"
	Focus {
		kind: String,
	},

	/// Generic escape hatch.
	Expression {
		expression: String,
	},
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Trigger {
	/// Simultaneous keys.
	///
	/// Cmd + Tab
	Chord { keys: Vec<Key> },

	/// One completed press/release.
	Tap { key: Key },

	/// Repeated taps.
	///
	/// Shift × 2
	Repeat {
		key: Key,
		count: u8,

		/// Time allowed between completed taps.
		#[serde(default)]
		max_interval_ms: Option<u64>,
	},

	/// Ordered triggers.
	///
	/// Cmd+K → Cmd+S
	Sequence {
		steps: Vec<Trigger>,

		/// This one matters much more than chord timeout.
		#[serde(default)]
		timeout_ms: Option<u64>,
	},

	/// Hold one key, then perform another trigger.
	///
	/// Hold Shift + Alt × 2
	HoldThen { held: Key, then: Box<Trigger> },

	/// Trigger while a key remains held.
	///
	/// Hold Shift + scroll
	WhileHeld { key: Key, trigger: Box<Trigger> },

	/// Explicit physical lifecycle.
	///
	/// ShiftRight ↓ → ShiftLeft ↓ → ShiftLeft ↑ → ShiftRight ↑
	Ordered { events: Vec<GestureEvent> },

	/// Mouse / scroll gestures.
	Pointer { trigger: PointerTrigger },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Key {
	Key(String),

	ShiftLeft,
	ShiftRight,

	ControlLeft,
	ControlRight,

	AltLeft,
	AltRight,

	MetaLeft,
	MetaRight,

	CapsLock,

	Enter,
	Tab,
	Escape,
	Space,
	Backspace,
	Delete,

	ArrowUp,
	ArrowDown,
	ArrowLeft,
	ArrowRight,

	F(u8),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureEvent {
	Down(Key),
	Up(Key),

	Tap(Key),

	MouseDown(MouseButton),
	MouseUp(MouseButton),

	Scroll { vertical: i64, horizontal: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PointerTrigger {
	Click {
		button: MouseButton,
	},

	DoubleClick {
		button: MouseButton,
	},

	Hold {
		button: MouseButton,
	},

	Scroll {
		axis: ScrollAxis,

		#[serde(default)]
		direction: Option<ScrollDirection>,
	},

	Drag {
		button: MouseButton,
	},

	Move,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MouseButton {
	Primary,
	Secondary,
	Middle,
	Button4,
	Button5,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScrollAxis {
	Vertical,
	Horizontal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScrollDirection {
	Positive,
	Negative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModifierKey {
	ShiftLeft,
	ShiftRight,
	ControlLeft,
	ControlRight,
	AltLeft,
	AltRight,
	MetaLeft,
	MetaRight,
	Fn,
	CapsLock,
}
