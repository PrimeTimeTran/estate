use crate::prelude::{traits::Ctx, *};

// https://www.youtube.com/watch?v=VwEV0UesMH0

impl<C> App<C>
where
	C: Ctx,
{
	pub fn context(self) -> Arc<C> {
		self.host.context()
	}

	pub fn init_services(&mut self) -> Result<()> {
		tracing::debug!("App init services");
		let handle = self.start_clock()?;
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			self.workers.push(handle);
			let handle = self.start_cargo_watcher()?;
			self.workers.push(handle);
			let handle = self.start_cursor_watcher_from_app()?;
			self.workers.push(handle);
		}
		tracing::debug!("App init services complete");
		Ok(())
	}

	pub fn run(&mut self) -> Result<()>
	where
		C::AppState: Send + Sync + 'static,
	{
		tracing::debug!("App run");
		self.init_services()?;
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			self.run_gui()?;
		}
		#[cfg(target_arch = "wasm32")]
		{
			// self.host.run()?;
		}

		Ok(())
	}

	pub fn shutdown(&mut self) {
		tracing::debug!("App shutdown");
		for worker in &self.workers {
			worker.stop();
		}
	}

	pub fn start_clock(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let clock = self.host.clock();
		let msg = String::from("App.start_clock.clock.run_background(Duration::from_secs(1));");
		Ok(clock.run_background(Duration::from_secs(1), msg))
	}

	pub fn start_clock_wasm(&mut self) {
		let clock = self.host.clock();
		// clock.inherent_background_tick(String::from(
		// 	"let clock = self.host.clock(); clock.inherent_background_tick",
		// ));
		let msg = String::from("start_clock_wasm clock.run_background(Duration::from_secs(1));");
		// clock.run_background(Duration::from_secs(1), msg.clone());
		// clock.run_background::<C, ()>(Duration::from_secs(1), msg.clone())
		// let handle = Clock::run_background(clock, Duration::from_secs(1), msg.clone());
	}

	pub fn worker(&self) -> &HostWorker<C> {
		self.host.worker()
	}
}
