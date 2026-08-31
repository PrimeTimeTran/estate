use crate::{
	app::{prelude::*, state::EstateState},
	prelude::*,
};

use crate::e;

pub struct App<R: Runtime> {
	pub(crate) engine: model::EstateEngine<R>,
	pub(crate) view: ViewType,
	events: tokio::sync::broadcast::Receiver<e::Event>,
}
impl<R: Runtime> App<R> {
	pub(crate) fn new(engine: model::EstateEngine<R>) -> Result<Self> {
		let events = engine.runtime.subscribe();

		Ok(Self {
			engine,
			events,
			view: ViewType::MarkdownView,
		})
	}
	pub fn update(&mut self) {
		while let Ok(event) = self.events.try_recv() {
			tracing::info!("🧭 App received update");
			match event.kind {
				e::EventKind::Navigate(view_type) => {
					tracing::info!("🧭 App received Navigate → {:?}", view_type);
					self.view = view_type;
				}
				_ => {}
			}
		}
	}
	pub fn runtime(&self) -> Arc<R> {
		Arc::clone(&self.engine.runtime)
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
			.emit(e::Event::app(e::EventKind::SessionStart));
	}
	pub fn new_task(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::TaskRequested {
				request: TaskRequest::Create(TaskKind::SyncBookmarks),
			}));
	}
	pub fn clear_tasks(&mut self) {
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::CommandExecuted {
				command: "task_clear".into(),
			}));
	}
	pub fn stop_session(&mut self) {
		println!("stop_session from app");
	}
}

impl<R: Runtime> App<R> {
	pub fn view(&self) -> ViewType {
		self.view
	}
	pub fn show_view(&mut self, view: ViewType) {
		self.view = view;
	}
	pub fn show_dashboard(&mut self) {
		self.show_view(ViewType::Dashboard);
	}
	pub fn show_tasks(&mut self) {
		self.show_view(ViewType::TaskManager);
		self
			.engine
			.runtime
			.emit(e::Event::app(e::EventKind::CommandExecuted {
				command: "task_list".into(),
			}));
	}
}
