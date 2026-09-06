use crate::prelude::*;

use tokio_util::sync::CancellationToken;

fn append_to_file(str: String) {}

fn sleep(str: String) {}

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

#[tokio::main]
pub async fn test_main() {
	let clock = HostClock;
	let worker = HostWorker;
	let handle = worker.run_background(move |cancel| async move {
		loop {
			tokio::select! {
					_ = cancel.cancelled() => {
							break;
					}
					_ = tokio::time::sleep(Duration::from_secs(1)) => {
							println!("background tick: {}", clock.now());
					}
			}
		}
	});
	tokio::time::sleep(Duration::from_secs(5)).await;
	handle.stop();
}

pub async fn test_main_2() {
	let clock = HostClock;
	let worker = HostWorker;

	let handle = worker.run_background(move |cancel| async move {
		loop {
			tokio::select! {
					_ = cancel.cancelled() => break,

					_ = tokio::time::sleep(Duration::from_secs(1)) => {
							println!("background tick: {}", clock.now());
					}
			}
		}
	});

	tokio::time::sleep(Duration::from_secs(5)).await;

	handle.stop();
}

impl App {
	pub fn new() -> Self {
		Self {
			host: Host::new(),
			handle_clock: None,
			handle_egui: None,
		}
	}
	pub fn run(&self, cli: Cli) -> Result<()> {
		Ok(())
	}
	pub fn start(&mut self) -> Result<()> {
		self.start_egui();
		self.start_clock();
		Ok(())
	}
	fn start_egui(&mut self) {
		let cancel = CancellationToken::new();
		// Install/register your egui hook here.
		//
		// The hook should retain `cancel.clone()` if it needs
		// to check for shutdown.

		self.handle_egui = Some(EguiHandle { cancel });
	}
	fn start_clock(&mut self) {
		let clock = self.host.clock();
		let handle = clock.run_background(Duration::from_secs(1));
		self.handle_clock = Some(handle)
	}
	pub fn shutdown(&mut self) {
		if let Some(handle) = self.handle_clock.take() {
			handle.stop();
		}

		if let Some(handle) = self.handle_egui.take() {
			handle.stop();
		}
	}
	pub fn clock(&self) -> &impl Clock {
		self.host.clock()
	}
	pub fn worker(&self) -> &impl Worker {
		self.host.worker()
	}
}

impl Clock for HostClock {
	fn now(&self) -> String {
		#[cfg(not(target_arch = "wasm32"))]
		{
			time_now()
		}

		#[cfg(target_arch = "wasm32")]
		{
			"0".to_string()
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

	fn run_background(&self, interval: Duration) -> ClockHandle {
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		#[cfg(not(target_arch = "wasm32"))]
		{
			std::thread::spawn(move || {
				let clock = HostClock;

				loop {
					if task_cancel.is_cancelled() {
						break;
					}

					clock.run_once();
					std::thread::sleep(interval);
				}
			});
		}
		#[cfg(target_arch = "wasm32")]
		{
			// Spawn async/browser task here later.
			let _ = (interval, task_cancel);
		}
		ClockHandle { cancel }
	}
}
impl EguiHandle {
	pub fn stop(&self) {
		self.cancel.cancel();
	}
	// winit different?
	// pub fn stop(self) {
	// 	// unregister hook
	// }
}

impl traits::Renderer for HostRenderer {
	#[cfg(target_arch = "wasm32")]
	fn render(&mut self) {
		// wasm rendering
	}
	#[cfg(not(target_arch = "wasm32"))]
	fn render(&mut self) {
		// native rendering
	}
}

impl WorkerHandle {
	pub fn stop(&self) {
		self.cancel.cancel();
	}
}

impl ClockHandle {
	pub fn stop(&self) {
		self.cancel.cancel();
	}
}

#[cfg(target_arch = "wasm32")]
impl Worker for HostWorker {
	type Handle = WorkerHandle;

	fn run_background<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) -> Fut + 'static,
		Fut: Future<Output = ()> + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		wasm_bindgen_futures::spawn_local(async move {
			task(task_cancel).await;
		});

		WorkerHandle { cancel }
	}

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + 'static,
	{
		task();
	}
}
#[cfg(not(target_arch = "wasm32"))]
impl Worker for HostWorker {
	type Handle = WorkerHandle;

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + Send + 'static,
	{
		loop {
			task();
		}
	}

	fn run_background<F, Fut>(&self, task: F) -> Self::Handle
	where
		F: FnOnce(CancellationToken) -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		tokio::spawn(async move {
			task(task_cancel).await;
		});

		WorkerHandle { cancel }
	}
}

pub struct App {
	pub host: Host,
	pub handle_clock: Option<ClockHandle>,
	// pub handle_clock: Option<<HostWorker as Worker>::Handle>,
	pub handle_egui: Option<EguiHandle>,
}
pub struct ClockHandle {
	cancel: CancellationToken,
}

pub struct EguiHandle {
	cancel: CancellationToken,
	// winit diff
	// Whatever is necessary to unregister the egui hook.
}
pub struct WorkerHandle {
	cancel: CancellationToken,
}
