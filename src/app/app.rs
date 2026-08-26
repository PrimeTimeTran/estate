use crate::app::*;

pub use modules::runtime::{ Runtime, RuntimeState };
pub use state::{ EstateState, StateStore };

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
