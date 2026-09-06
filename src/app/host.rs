// Platform-specific concrete implementations,
// selected at compilation.

use std::time::{SystemTime, UNIX_EPOCH};

trait Clock {
	fn now(&self) -> u64;
}

struct HostClock;

impl Clock for HostClock {
	#[cfg(target_arch = "wasm32")]
	fn now(&self) -> u64 {
		// WASM implementation
		// Replace with JS/browser time later.
		0
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn now(&self) -> u64 {
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs()
	}
}

trait Worker {
	fn spawn(&self, task: Box<dyn FnOnce() + Send>);
}

struct HostWorker;

trait Worker {
	fn spawn(&self, task: Box<dyn FnOnce() + Send>);
}

struct HostWorker;

fn task() {
	println!("it's currently.")
}

impl Worker for HostWorker {
	#[cfg(target_arch = "wasm32")]
	fn spawn(&self, task: Box<dyn FnOnce() + Send>) {
		// Later:
		// wasm_bindgen_futures::spawn_local(async move {
		//     task();
		// });
		println!("WASM worker");
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn spawn(&self, task: Box<dyn FnOnce() + Send>) {
		std::thread::spawn(move || {
			task();
		});
	}
}
trait Renderer {
	fn render(&mut self);
}

struct HostRenderer;

impl Renderer for HostRenderer {
	#[cfg(target_arch = "wasm32")]
	fn render(&mut self) {
		// wasm rendering
	}

	#[cfg(not(target_arch = "wasm32"))]
	fn render(&mut self) {
		// native rendering
	}
}

trait Provide {
	type Clock: Clock;
	type Worker: Worker;
	// type Renderer: Renderer;

	fn clock(&self) -> &Self::Clock;
	fn worker(&self) -> &Self::Worker;
	// fn renderer(&mut self) -> &mut Self::Renderer;
}

pub struct Host {
	worker: HostWorker,
	clock: HostClock,
	// renderer: HostRenderer,
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

struct App {
	host: Host,
}

impl App {
	fn new() -> Self {
		Self { host: Host::new() }
	}

	fn clock(&self) -> &impl Clock {
		self.host.clock()
	}

	fn worker(&self) -> &impl Worker {
		self.host.worker()
	}
}

fn main() {
	let app = App::new();

	println!("Time: {}", app.clock().now());

	app.worker().spawn(Box::new(|| {
		println!("Hello from worker!");
	}));

	std::thread::sleep(std::time::Duration::from_millis(100));
}
