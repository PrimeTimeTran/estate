use crate::prelude::*;

#[cfg(target_arch = "wasm32")]
impl<C> WorkHandle<C, std::thread::JoinHandle<()>>
where
	C: Ctx,
{
	pub fn join(self) -> std::thread::Result<()> {
		self.join()
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl<C> WorkHandle<C, tokio::task::JoinHandle<()>>
where
	C: Ctx,
{
	// pub async fn join(self) -> Result<JoinHandle<()>> {
	// Ok(self.join)
	// }
	pub async fn join(self) -> Result<()> {
		self.join.await?;
		Ok(())
	}
}
impl<C, J> WorkHandle<C, J>
where
	C: Ctx,
{
	#[cfg(not(target_arch = "wasm32"))]
	pub fn new(cancel: CancellationToken, join: J) -> Self {
		Self {
			cancel,
			join,
			_ctx: PhantomData,
		}
	}

	#[cfg(target_arch = "wasm32")]
	pub fn new(cancel: CancellationToken) -> Self {
		Self {
			cancel,
			_ctx: PhantomData,
			_phantom: PhantomData,
		}
	}

	pub fn stop(&self) {
		self.cancel.cancel();
	}
}

pub struct WorkerSupervisor<C, S>
where
	C: Ctx,
{
	workers: HashMap<WorkerId, Worker<C, S>>,
}
// pub enum WorkerId {
// MacosHid,
// CargoWatcher,
// ActiveApp,
// Cursor,
// Lsp,
// Daemon,
// }
pub struct WorkerId(pub &'static str);
pub struct Worker<C, S>
where
	C: Ctx,
{
	pub name: String,
	pub handle: WorkHandle<C, S>,
	pub status: WorkerStatus,
}
pub enum WorkerStatus {
	Starting,
	Running,
	Stopping,
	Stopped,
	Failed(String),
}
pub struct WorkerInfo {
	pub id: WorkerId,
	pub name: String,
	pub status: WorkerStatus,
	pub started_at: Option<Instant>,
}
impl<C, S> WorkerSupervisor<C, S>
where
	C: Ctx,
{
	pub fn new() -> Self {
		Self {
			workers: HashMap::new(),
		}
	}

	pub fn spawn<F>(&mut self, worker: F)
	where
		F: Future<Output = ()> + Send + 'static,
	{
		// register worker
	}

	pub async fn shutdown(self) {
		// self.cancel.cancel();
		// for worker in self.workers.values() {
		// worker.stop();
		// }
		// for worker in self.workers {
		// let _ = worker.join().await;
		// }
	}
}

/// "This is a unit of work that I know how to stop."
///
pub struct WorkHandle<C, J>
where
	C: Ctx,
{
	pub cancel: CancellationToken,
	_ctx: PhantomData<C>,
	#[cfg(not(target_arch = "wasm32"))]
	pub join: J,
	#[cfg(target_arch = "wasm32")]
	_phantom: PhantomData<J>,
}

#[cfg(target_arch = "wasm32")]
pub type PlatformJoin = ();

#[cfg(not(target_arch = "wasm32"))]
pub type ClockWork<NativeContext> = WorkHandle<NativeContext, tokio::task::JoinHandle<()>>;

#[cfg(target_arch = "wasm32")]
pub type ClockWork<WebContext> = WorkHandle<WebContext, ()>;
