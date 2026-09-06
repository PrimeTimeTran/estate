use crate::prelude::*;

pub trait Clock {
	fn now(&self) -> String;

	/// Run once and return.
	fn run_once(&self);

	/// Run repeatedly in the foreground.
	fn run_foreground(&self, interval: Duration);

	/// Spawn the clock as a background process/task.
	fn run_background(&self, interval: Duration);
}

pub struct HostClock;

impl Clock for HostClock {
	fn now(&self) -> String {
		#[cfg(target_arch = "wasm32")]
		{
			// Replace with JS/browser time later.
			0
		}

		#[cfg(not(target_arch = "wasm32"))]
		{
			use chrono::Utc;
			let now = Utc::now();
			let format = "%B %-d, %Y at %-I:%M:%S %p UTC";
			// println!(now.format(format).to_string());
			now.format(format).to_string()
			// SystemTime::now()
			// 	.duration_since(UNIX_EPOCH)
			// 	.unwrap()
			// 	.as_secs()
		}
	}

	fn run_once(&self) {
		println!("tick: {}", self.now());
	}

	fn run_foreground(&self, interval: Duration) {
		loop {
			self.run_once();
			std::thread::sleep(interval);
		}
	}

	fn run_background(&self, interval: Duration) {
		#[cfg(not(target_arch = "wasm32"))]
		{
			std::thread::spawn(move || {
				let clock = HostClock;

				loop {
					clock.run_once();
					std::thread::sleep(interval);
				}
			});
		}

		#[cfg(target_arch = "wasm32")]
		{
			// Later: window.setInterval(...) / async task.
			let _ = interval;
			println!("WASM background clock");
		}
	}
}

pub trait Worker {
	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static;

	fn run_background<F>(&self, task: F)
	where
		F: Fn() + Send + 'static;
}

fn task() {
	println!("it's currently.")
}
pub struct HostWorker;

impl Worker for HostWorker {
	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static,
	{
		loop {
			task();
		}
	}

	fn run_background<F>(&self, task: F)
	where
		F: Fn() + Send + 'static,
	{
		std::thread::spawn(task);
	}
}
// impl Worker for HostWorker {
// 	#[cfg(target_arch = "wasm32")]
// 	fn spawn(&self, task: Box<dyn FnOnce() + Send>) {
// 		// Later:
// 		// wasm_bindgen_futures::spawn_local(async move {
// 		//     task();
// 		// });
// 		println!("WASM worker");
// 	}

// 	#[cfg(not(target_arch = "wasm32"))]
// 	fn spawn(&self, task: Box<dyn FnOnce() + Send>) {
// 		std::thread::spawn(move || {
// 			task();
// 		});
// 	}
// }
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

impl Host {
	fn new() -> Self {
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

// fn main() {
// 	let app = App::new();

// 	println!("Time: {}", app.clock().now());

// 	let mut count = 0;

// 	app.worker().spawn(Box::new(|| {
// 		println!("Hello from worker!");
// 	}));

// 	std::thread::sleep(std::time::Duration::from_millis(100));
// }

fn append_to_file(str: String) {}
fn sleep(str: String) {}
fn main() {
	let clock = HostClock;
	let worker = HostWorker;

	worker.run_background(move || {
		loop {
			println!("background tick: {}", clock.now());
			std::thread::sleep(Duration::from_secs(1));
		}
	});

	// worker.run_background(|| {
	// 	loop {
	// 		let now = clock.now();

	// 		append_to_file(now);

	// 		sleep(Duration::from_secs(1));
	// 	}
	// });

	loop {
		println!("foreground");
		std::thread::sleep(Duration::from_secs(5));
	}
}
