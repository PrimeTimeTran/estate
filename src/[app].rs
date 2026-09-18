//! The [crate::App]
//!
use crate::prelude::*;

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
		self.init_api()?;

		let handle = self.start_clock()?;
		self.workers.push(handle);
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		{
			let handle = self.start_cargo_watcher()?;
			self.workers.push(handle);
			let handle = self.start_cursor_watcher_from_app()?;
			self.workers.push(handle);
		}
		tracing::debug!("App init services complete");
		Ok(())
	}
	pub fn init_api(&mut self) -> Result<()> {
		tracing::info!("Connecting API");

		let context = self.host.context();
		let api = context.api();

		tracing::info!(
			context = format_args!("{:p}", Arc::as_ptr(&context)),
			api = format_args!("{:p}", api),
			"API instance"
		);

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
		let _clock = self.host.clock();
		// clock.inherent_background_tick(String::from(
		// 	"let clock = self.host.clock(); clock.inherent_background_tick",
		// ));
		let _msg = String::from("start_clock_wasm clock.run_background(Duration::from_secs(1));");
		// clock.run_background(Duration::from_secs(1), msg.clone());
		// clock.run_background::<C, ()>(Duration::from_secs(1), msg.clone())
		// let handle = Clock::run_background(clock, Duration::from_secs(1), msg.clone());
	}

	pub fn worker(&self) -> &HostWorker<C> {
		self.host.worker()
	}
}
