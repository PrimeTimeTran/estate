pub trait Monitor {
	fn watch(&mut self);
	fn rx(&mut self);
	fn poll(&mut self) -> bool;
}

#[cfg(target_arch = "wasm32")]
pub struct WebMonitor;

#[cfg(target_arch = "wasm32")]
impl Monitor for WebMonitor {
	fn watch(&mut self) {}

	fn rx(&mut self) {}

	fn poll(&mut self) -> bool {
		false
	}
}
