use winit::event_loop::EventLoopProxy;

pub use crate::native::{monitor::NativeMonitor, prelude::*, state::NativeStateStore};
use crate::{
	app::{
		Runtime,
		state::{EstateState, StateStore},
	},
	e,
};
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct NativeRuntime {
	event_rx: Arc<Mutex<broadcast::Receiver<e::Event>>>,
	handle: tokio::runtime::Handle,
	proxy: Arc<Mutex<Option<EventLoopProxy<AppEvent>>>>,
	pub events: EventBus,
	pub session: Session,
	pub state: Arc<RuntimeState>,
	pub store: NativeStateStore,
	pub tasks: Arc<RwLock<TaskManager>>,
}

impl NativeRuntime {
	pub fn new(handle: tokio::runtime::Handle) -> Result<Self> {
		let store = NativeStateStore::new()?;
		let state = store.load()?;
		let runtime_state = RuntimeState::new(state);
		let events = EventBus::new();
		let event_rx = Arc::new(Mutex::new(events.subscribe()));
		Ok(Self {
			event_rx,
			events,
			handle,
			proxy: Arc::new(Mutex::new(None)),
			session: Session::default(),
			state: Arc::new(runtime_state),
			store,
			tasks: Arc::new(RwLock::new(TaskManager::new())),
		})
	}

	pub fn attach_event_proxy(&self, proxy: EventLoopProxy<AppEvent>) {
		*self.proxy.lock().unwrap() = Some(proxy);
	}

	pub fn event_processed(&self) {
		let mut state = self.state.write();
		state.events_processed += 1;
	}

	pub fn start_services(&self) {
		println!("NativeRuntime start_services");
		tracing::info!("NativeRuntime start_services")
	}
}

impl Runtime for NativeRuntime {
	type EventReceiver = NativeEventReceiver;

	fn spawn<F>(&self, future: F)
	where
		F: std::future::Future<Output = ()> + Send + 'static,
	{
		self.handle.spawn(future);
	}

	async fn sleep(&self, duration: std::time::Duration) {
		tokio::time::sleep(duration).await;
	}

	fn emit(&self, event: e::Event) {
		tracing::debug!("NativeRuntime {:?}", event.kind.clone());
		self.events.emit(event);
		if let Some(proxy) = self.proxy.lock().unwrap().as_ref() {
			let _ = proxy.send_event(AppEvent::RuntimeEvent);
		}
	}

	fn state(&self) -> &RuntimeState {
		&self.state
	}

	fn save(&self, state: &EstateState) -> Result<()> {
		self.store.save(state)
	}

	fn session(&self) -> Session {
		self.session.clone()
	}

	fn subscribe(&self) -> Self::EventReceiver {
		NativeEventReceiver {
			rx: self.events.subscribe(),
		}
	}

	fn try_recv(&self) -> Option<e::Event> {
		self.event_rx.lock().unwrap().try_recv().ok()
	}

	fn start_dispatcher(self: &Arc<Self>) {
		let runtime = Arc::clone(self);
		let handle = runtime.handle.clone();
		let mut receiver = runtime.events.subscribe();
		let mut dispatcher = EventDispatcher::new();

		dispatcher.register(crate::event::handler::TaskHandler);
		dispatcher.register(crate::event::handler::StateHandler);
		dispatcher.register(crate::event::handler::CommandHandler);
		dispatcher.register(crate::event::handler::FileWatcherHandler);
		dispatcher.register(crate::event::handler::AppHandler);
		dispatcher.register(crate::event::handler::NavigationHandler);

		handle.spawn(async move {
			loop {
				match receiver.recv().await {
					Ok(event) => {
						tracing::debug!("🔥 native::runtime::dispatcher {:?}", event.kind);

						dispatcher.dispatch(event, &runtime).await;
					}
					Err(broadcast::error::RecvError::Lagged(count)) => {
						tracing::warn!(count, "native::runtime::start_dispatcher lagged");
					}
					Err(broadcast::error::RecvError::Closed) => {
						tracing::warn!("native::runtime::start_dispatcher closed");
						break;
					}
				}
			}
		});
	}
}
pub struct NativeAppContext<'a> {
	pub base: AppContext<'a, NativeRuntime>,
	pub monitor: &'a mut NativeMonitor,
}
impl<'a> NativeAppContext<'a> {
	pub fn state(&self) -> std::sync::RwLockReadGuard<'_, EstateState> {
		self.base.state()
	}
	#[cfg(not(target_arch = "wasm32"))]
	pub fn poll_state(&mut self) -> bool {
		todo!("Unused for not.")
	}
}
pub struct NativeEventReceiver {
	pub rx: broadcast::Receiver<e::Event>,
}
impl EventReceiver for NativeEventReceiver {
	fn try_recv(&mut self) -> Option<e::Event> {
		self.rx.try_recv().ok()
	}
}
// impl AppHost<NativeRuntime> for NativeApp {
// 	fn app(&mut self) -> &mut App<NativeRuntime> {
// 		&mut self.app
// 	}
// }

impl Drop for NativeRuntime {
	fn drop(&mut self) {
		tracing::info!("💀 NativeRuntime DROPPED");
	}
}
