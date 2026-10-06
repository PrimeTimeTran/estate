use std::collections::HashSet;

pub struct KeyTracker {
	pressed: HashSet<u32>,
}

impl KeyTracker {
	pub fn new() -> Self {
		Self {
			pressed: HashSet::new(),
		}
	}

	pub fn on_key_down(&mut self, key: u32) {
		if self.pressed.insert(key) {
			println!("{key} DOWN");
		}
	}

	pub fn on_key_up(&mut self, key: u32) {
		if self.pressed.remove(&key) {
			println!("{key} UP");
		}
	}
}
