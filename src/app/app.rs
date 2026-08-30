use crate::{
	app::{event::EventKind, model, state::EstateState},
	prelude::*,
};

pub struct App<R: Runtime> {
	pub(crate) engine: model::EstateEngine<R>,
}
impl<R: Runtime> App<R> {
	pub(crate) fn new(engine: model::EstateEngine<R>) -> Result<Self> {
		Ok(Self { engine })
	}
	pub fn runtime(&self) -> &R {
		&self.engine.runtime
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
	pub fn on_start(&mut self) {
		self
			.engine
			.runtime
			.emit(Event::app(EventKind::SessionStart));
	}
	pub fn new_task(&mut self) {
		self
			.engine
			.runtime
			.emit(Event::app(EventKind::TaskRequested {
				request: TaskRequest::Create(TaskKind::SyncBookmarks),
			}));
	}
	pub fn show_tasks(&mut self) {
		self
			.engine
			.runtime
			.emit(Event::app(EventKind::CommandExecuted {
				command: "task_list".into(),
			}));
	}
	pub fn clear_tasks(&mut self) {
		self
			.engine
			.runtime
			.emit(Event::app(EventKind::CommandExecuted {
				command: "task_clear".into(),
			}));
	}
	pub fn stop_session(&mut self) {
		println!("stop_session from app");
	}
}
