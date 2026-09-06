use crate::prelude::*;

impl Host {
	pub fn new() -> Self {
		Self {
			clock: HostClock,
			worker: HostWorker::new(),
		}
	}
	pub fn run(&self) -> Result<()> {
		tracing::info!("Host run");
		// Host-level execution goes here.
		// This does NOT need to be winit.
		// CLI, daemon, or GUI can build on top of this.
		Ok(())
	}

	pub fn clock(&self) -> &HostClock {
		&self.clock
	}

	pub fn worker(&self) -> &HostWorker {
		&self.worker
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
impl HostWorker {
	pub fn new() -> Self {
		let runtime = tokio::runtime::Runtime::new().unwrap();
		Self {
			runtime: Arc::new(runtime),
		}
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
	fn run_background<F, Fut>(&self, task: F) -> WorkerHandle
	where
		F: FnOnce(CancellationToken) -> Fut + Send + 'static,
		Fut: Future<Output = ()> + Send + 'static,
	{
		let cancel = CancellationToken::new();
		let task_cancel = cancel.clone();

		self.runtime.spawn(async move {
			task(task_cancel).await;
		});

		WorkerHandle { cancel }
	}

	// fn run_background<F, Fut>(&self, task: F) -> Self::Handle
	// where
	// 	// So Tokio can move the entire future between worker threads.
	// 	F: FnOnce(CancellationToken) -> Fut + Send + 'static,
	// 	// Enforced for the subsequent move
	// 	Fut: Future<Output = ()> + Send + 'static,
	// {
	// 	let cancel = CancellationToken::new();
	// 	let task_cancel = cancel.clone();

	// 	// requires the spawned future to be:
	// 	// Future + Send + 'static
	// 	tokio::spawn(async move {
	// 		task(task_cancel).await;
	// 	});

	// 	WorkerHandle { cancel }
	// }
}

pub struct HostWorker {
	pub runtime: Arc<tokio::runtime::Runtime>,
}
pub struct HostClock;
pub struct HostRenderer;
pub struct Host {
	worker: HostWorker,
	clock: HostClock,
	// renderer: HostRenderer,
}
pub struct WorkerHandle {
	pub cancel: CancellationToken,
}
