pub struct Size {
	pub value: f32,
	pub min: f32,
	pub max: f32,
	pub resizable: bool,
}
impl Size {
	pub fn new(value: f32, min: f32, max: f32) -> Self {
		Self {
			value: value.clamp(min, max),
			min,
			max,
			resizable: true,
		}
	}
	pub fn set(&mut self, value: f32) {
		self.value = value.clamp(self.min, self.max);
	}
	pub fn resize(&mut self, delta: f32) {
		self.set(self.value + delta);
	}
}

#[derive(Clone, Copy)]
pub enum ResizeEdge {
	Left,
	Right,
	Top,
	Bottom,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CursorTarget {
	ActivityBar,
	DockLeft,
	#[default]
	Main,
	PrimaryBar,
	SecondaryBar,
	BottomPanel,
	StatusBar,
	DockRight,
	None,
}

impl CursorTarget {
	pub fn name(self) -> &'static str {
		match self {
			Self::ActivityBar => "Activity Bar",
			Self::DockLeft => "Dock Left",
			Self::Main => "Main",
			Self::PrimaryBar => "Primary Bar",
			Self::SecondaryBar => "Secondary Bar",
			Self::BottomPanel => "Bottom Panel",
			Self::StatusBar => "Status Bar",
			Self::DockRight => "Dock Right",
			Self::None => "Nothing",
		}
	}
}
