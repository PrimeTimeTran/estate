use crate::share::prelude::*;
use std::collections::VecDeque;

pub(crate) mod host;
pub(crate) mod job;
pub(crate) mod model;
pub(crate) mod modules;

#[path = "modules/monitor.rs"]
pub(crate) mod monitor;

#[cfg(not(target_arch = "wasm32"))]
#[path = "modules/monitor_native.rs"]
pub(crate) mod monitor_native;

pub(crate) mod state;

pub use job::*;
pub use modules::runtime::{ Runtime, RuntimeState };
pub use state::{ EstateState, NativeStateStore, StateStore };

// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

pub struct App<R: Runtime> {
	pub engine: model::EstateEngine<R>,
}
impl<R: Runtime> App<R> {
	pub fn new(engine: model::EstateEngine<R>) -> anyhow::Result<Self> {
		Ok(Self { engine })
	}
}

impl<R: Runtime> App<R> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.engine.runtime.state().read()
	}

	pub fn jobs(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.state()
	}
}

impl<R: Runtime> App<R> {
	pub fn new_task(&mut self) {
		self.engine.runtime.emit(
			Event::app(EventKind::TaskRequested {
				request: TaskRequest::Create(TaskKind::SyncBookmarks),
			})
		);
	}

	pub fn show_tasks(&mut self) {
		self.engine.runtime.emit(
			Event::app(EventKind::CommandExecuted {
				command: "task_list".into(),
			})
		);
	}

	pub fn clear_tasks(&mut self) {
		self.engine.runtime.emit(
			Event::app(EventKind::CommandExecuted {
				command: "task_clear".into(),
			})
		);
	}
}

#[cfg(feature = "native")]
pub(crate) mod context;
#[cfg(feature = "native")]
pub use context::*;

#[cfg(feature = "native")]
pub(crate) mod state_native;
#[cfg(feature = "native")]
pub use state_native::*;
