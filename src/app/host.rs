use crate::prelude::*;

pub struct HostWorker;
pub struct HostClock;
pub struct HostRenderer;
pub struct Host {
	pub worker: HostWorker,
	pub clock: HostClock,
	// renderer: HostRenderer,
}

impl Host {
	pub fn new() -> Self {
		Self {
			clock: HostClock,
			worker: HostWorker,
		}
	}
}

impl Provide for Host {
	type Clock = HostClock;
	type Worker = HostWorker;
	// type Renderer = HostRenderer;
	fn clock(&self) -> &Self::Clock {
		&self.clock
	}
	fn worker(&self) -> &Self::Worker {
		&self.worker
	}
	// fn renderer(&mut self) -> &mut Self::Renderer {
	// 	&mut self.renderer
	// }
}
