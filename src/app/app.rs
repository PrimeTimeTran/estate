use crate::prelude::*;
// https://github.com/rust-lang/rust/issues/41517
// https://github.com/rust-lang/rust/issues/55628
// https://github.com/rust-lang/rfcs/pull/1733

fn task() {
	println!(
		"The task asts the time that's used by the clock. {}",
		time_now()
	)
}

fn time_now() -> String {
	use chrono::Utc;
	let now = Utc::now();
	let format = "%B %-d, %Y at %-I:%M:%S %p UTC";
	now.format(format).to_string()
}

impl Clock for HostClock {
	fn now(&self) -> String {
		#[cfg(target_arch = "wasm32")]
		{
			// Replace with JS/browser time later.
			0
		}

		#[cfg(not(target_arch = "wasm32"))]
		{
			time_now()
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

impl Worker for HostWorker {
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

impl crate::r#trait::Renderer for HostRenderer {
	#[cfg(target_arch = "wasm32")]
	fn render(&mut self) {
		// wasm rendering
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn render(&mut self) {
		// native rendering
	}
}

fn sleep(str: String) {}
fn append_to_file(str: String) {}

pub fn test_main() {
	let clock = HostClock;
	let worker = HostWorker;

	worker.run_background(move || {
		loop {
			println!("background tick: {}", clock.now());
			std::thread::sleep(Duration::from_secs(1));
		}
	});

	loop {
		// println!("foreground");
		task();
		std::thread::sleep(Duration::from_secs(3));
	}
	// worker.run_background(|| {
	// 	loop {
	// 		let now = clock.now();

	// 		append_to_file(now);

	// 		sleep(Duration::from_secs(1));
	// 	}
	// });
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

pub struct App {
	pub host: Host,
}

impl App {
	pub fn new() -> Self {
		Self { host: Host::new() }
	}
	pub fn run(&self, cli: Cli) -> Result<()> {
		Ok(())
	}
	pub fn clock(&self) -> &impl Clock {
		self.host.clock()
	}
	pub fn worker(&self) -> &impl Worker {
		self.host.worker()
	}
}
