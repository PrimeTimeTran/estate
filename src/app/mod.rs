use crate::{ share::prelude::* };

pub(crate) mod modules;
pub(crate) mod model;

pub use modules::runtime::Runtime;

// Generic traits should exist inside of ./src/app module
// Platform implementations for native, mobile, web should exist in their own respective namespaces
// ./src/native
// ./src/web
// ./src/mobile

#[derive(Clone)]
pub struct App<R: Runtime> {
	pub engine: model::EstateEngine<R>,
}
impl<R: Runtime> App<R> {
	pub fn new(engine: model::EstateEngine<R>) -> Self {
		Self { engine }
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

#[cfg(not(target_arch = "wasm32"))]
pub struct AppContext<'a> {
	pub app: &'a mut App<crate::native::app::NativeRuntime>,
}
