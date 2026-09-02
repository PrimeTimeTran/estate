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
