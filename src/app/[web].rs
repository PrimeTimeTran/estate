use crate::{
	doc,
	prelude::{traits::Ctx, *},
	ui, ui_prelude as gui,
};

impl Ctx for WebContext {
	type State = WebState;
	fn state(&self) -> &Self::State {
		self.state()
	}
}

impl Host<WebContext> {
	// pub fn new(context: Arc<C>) -> anyhow::Result<Self> {
	// 	let clock = HostClock {};
	// 	Ok(Self {
	// 		clock,
	// 		context,
	// 		worker: HostWorker::new(),
	// 	})
	// }
	pub fn init() -> Result<Self> {
		Self::new(Arc::new(WebContext::default()))
	}
}

impl<WebContext> Host<WebContext>
where
	WebContext: Ctx,
{
	pub fn clock(&self) -> &HostClock {
		&self.clock
	}
	pub fn context(&self) -> Arc<WebContext> {
		self.context.clone()
	}
	pub fn wait_for_shutdown(&self) {
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			self.worker.block_on(async {
				tokio::signal::ctrl_c()
					.await
					.expect("failed to listen for Ctrl+C");

				tracing::info!("Ctrl+C received");
			});
		}
	}
	pub fn worker(&self) -> &HostWorker<WebContext> {
		&self.worker
	}
}
impl<C> Worker<C> for HostWorker<C>
where
	C: Ctx,
{
	type Handle = WorkHandle<C, tokio::task::JoinHandle<()>>;

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

		WorkHandle::new(cancel)
	}

	fn run_foreground<F>(&self, task: F)
	where
		F: Fn() + 'static,
	{
		task();
	}
}

#[derive(Default)]
pub struct WebContext;

#[derive(Clone, Debug, Default)]
pub struct WebState;
