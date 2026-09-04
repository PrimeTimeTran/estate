pub use crate::native::{monitor::NativeMonitor, prelude::*, state::NativeStateStore};
use crate::{
	EventReceiver,
	app::{
		Runtime,
		state::{EstateState, StateStore},
	},
	e,
};
use std::sync::Mutex;
use winit::event_loop::EventLoopProxy;

#[derive(Clone, Debug)]
pub struct NativeRuntime {
	event_rx: Arc<Mutex<broadcast::Receiver<e::Event>>>,
	pub handle: tokio::runtime::Handle,
	proxy: Arc<Mutex<Option<EventLoopProxy<AppEvent>>>>,
	pub events: EventBus,
	pub session: Session,
	pub state: Arc<RuntimeState>,
	pub store: NativeStateStore,
	pub tasks: Arc<RwLock<TaskManager>>,
	pub state_service: Arc<StateService>,
	// pub api: Box<dyn Api>,
	pub session_service: Arc<SessionService>,
	pub executor: NativeExecutor,
	services: NativeServices,
}

impl NativeRuntime {
	pub async fn new(handle: tokio::runtime::Handle) -> anyhow::Result<Self> {
		let store = NativeStateStore::new()?;
		let state_service = Arc::new(StateService::new(crate::STATE_PATH));
		let session_service = Arc::new(SessionService::new(Arc::clone(&state_service)));
		let state = store.load()?;
		let runtime_state = RuntimeState::new(state);
		let events = EventBus::new();
		let event_rx = Arc::new(Mutex::new(events.subscribe()));
		let services = NativeServices::connect().await?;

		let executor = NativeExecutor {
			handle: handle.clone(),
		};
		Ok(Self {
			event_rx,
			events,
			handle,
			services,
			// api: Box::new(NativeApiClient::new()),
			proxy: Arc::new(Mutex::new(None)),
			session: Session::default(),
			state: Arc::new(runtime_state),
			store,
			tasks: Arc::new(RwLock::new(TaskManager::new())),
			state_service,
			session_service,
			executor,
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

#[derive(Clone, Debug)]
pub struct NativeExecutor {
	pub handle: tokio::runtime::Handle,
}
// ============================================================
// INHERENT METHOD
// ============================================================
//
// This method belongs directly to the concrete `NativeExecutor`
// type.
//
// It is NOT a trait implementation.
//
// Technical name:
//   "inherent method" / "inherent impl"
//
// Called when Rust has a concrete `NativeExecutor` value and
// method resolution selects this method.
//
// Example:
//
//   let executor: NativeExecutor = ...;
//   executor.spawn(future);
//
// Because `spawn` exists directly on `NativeExecutor`, this
// inherent method takes precedence over a trait method with the
// same name when the receiver's concrete type is known.
//
// This is useful for functionality that is specifically owned
// by the concrete type and doesn't need to participate in a
// generic trait abstraction.
//
impl NativeExecutor {
	pub fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + Send + 'static,
	{
		println!("🔥 NativeExecutor::spawn CALLED");
		// Nothing is actually spawned here yet.
		//
		// This is intentionally left as a case study.
		//
		// If we uncommented this:
		//
		// tokio::spawn(async move {
		// 	println!("🔥 NativeRuntime task STARTED");
		// 	future.await;
		// });
		//
		// this would be a concrete NativeExecutor-specific
		// implementation.
	}
}

// ============================================================
// TRAIT IMPLEMENTATION: NativeExecutor -> Executor
// ============================================================
//
// This says:
//
//   "NativeExecutor satisfies the generic `Executor` contract."
//
// Technical name:
//   "trait implementation"
//   "`Executor` implementation for `NativeExecutor`"
//
// This is what allows generic code to say:
//
//   fn start<E: Executor>(executor: E) {
//       executor.spawn(future);
//   }
//
// without knowing that `E` is actually `NativeExecutor`.
//
// IMPORTANT:
//
// This `spawn` is a DIFFERENT method from the inherent
// `NativeExecutor::spawn` above.
//
// It has the same name because the `Executor` trait requires
// a method called `spawn`.
//
// The implementation is selected when the call is being made
// through the `Executor` abstraction.
//
// In other words:
//
//   NativeExecutor
//       │
//       └── implements Executor
//                   │
//                   └── Executor::spawn()
//
impl Executor for NativeExecutor {
	fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
		println!("🔥 Executor for NativeExecutor::spawn CALLED");

		// NativeExecutor owns the Tokio Handle.
		//
		// Therefore this is the actual bridge between our
		// platform-independent `Executor` abstraction and Tokio.
		self.handle.spawn(async move {
			println!("🔥 Executor for NativeExecutor::spawn moving");
			future.await;
		});
	}
}

// ============================================================
// TRAIT IMPLEMENTATION: NativeRuntime -> Executor
// ============================================================
//
// This is a SECOND relationship.
//
// Here we are saying:
//
//   "NativeRuntime ALSO satisfies the Executor contract."
//
// Therefore generic code like:
//
//   fn foo<E: Executor>(executor: E) {
//       executor.spawn(future);
//   }
//
// could receive either:
//
//   NativeExecutor
//
// OR:
//
//   NativeRuntime
//
// and both are valid `E` types.
//
// They are different concrete types implementing the same trait.
//
// Conceptually:
//
//                    Executor
//                    /      \
//                   /        \
//       NativeExecutor    NativeRuntime
//              │               │
//              ▼               ▼
//         Tokio Handle      Tokio runtime
//
//
// IMPORTANT:
//
// This does NOT mean:
//
//   NativeRuntime::spawn()
//          automatically calls
//   NativeExecutor::spawn()
//
// There is no automatic delegation.
//
// These are two independent implementations of the same trait.
//
impl Executor for NativeRuntime {
	fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
		println!("✅ Executor for NativeRuntime");
		tokio::spawn(future);
	}
	// This implementation chooses to use Tokio directly.
	//
	// Notice that it does NOT use:
	//
	//   self.executor.spawn(...)
	//
	// even though NativeRuntime may contain a NativeExecutor.
	//
	// That would be explicit delegation, which is a different
	// architectural choice.
}

// ============================================================
// TRAIT IMPLEMENTATION: NativeRuntime -> Runtime
// ============================================================
//
// This is yet ANOTHER relationship.
//
// NativeRuntime satisfies the `Runtime` trait.
//
// If `Runtime` contains a method named `spawn`, then
// `NativeRuntime` must provide an implementation of that
// contract.
//
// Technical name:
//   "trait method implementation"
//
// This `spawn` is NOT the same method as:
//
//   NativeExecutor::spawn
//
// and NOT the same method as:
//
//   Executor::spawn
//
// It merely has the same method name.
//
impl Runtime for NativeRuntime {
	fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		println!("✅ NativeRuntime::spawn");
	}
	// This is currently just a demonstration.
	//
	// Notice that Runtime::spawn has a different contract:
	//
	//     Future + 'static
	//
	// while native Executor::spawn requires:
	//
	//     Future + Send + 'static
	//
	// That difference matters enormously when generic code
	// tries to move the future onto a native multithreaded
	// executor.
	// --------------------------------------------------------
	// Another Runtime capability
	// --------------------------------------------------------
	//
	// Runtime isn't necessarily "the thing that spawns".
	//
	// It can expose platform capabilities that application code
	// needs without exposing Tokio or WASM implementation details.
	//
	// Native:
	//
	//     tokio::time::sleep()
	//
	// WASM:
	//
	//     gloo_timers::future::sleep()
	//
	// Application code can simply say:
	//
	//     runtime.sleep(duration).await;
	//
	// without caring which platform it is running on.
	//
	// fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send {
	// 	tokio::time::sleep(duration)
	// }
	//

	fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send {
		tokio::time::sleep(duration)
	}

	type EventReceiver = NativeEventReceiver;

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
	fn event_processed(&self) {
		println!("event processed")
	}
	fn tasks(&self) -> &Arc<RwLock<TaskManager>> {
		&self.tasks
	}
	fn start_dispatcher(self: &Arc<Self>) {
		let runtime = Arc::clone(self);
		let handle = runtime.handle.clone();
		let mut receiver = runtime.events.subscribe();
		let mut dispatcher = EventDispatcher::<NativeRuntime>::new();
		dispatcher.register(TaskHandler);
		dispatcher.register(StateHandler);
		dispatcher.register(CommandHandler);
		dispatcher.register(FileWatcherHandler);
		dispatcher.register(NavigationHandler);
		dispatcher.register(AppHandler);
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
	fn state_service(&self) -> &Arc<StateService> {
		&self.state_service
	}
	fn session_service(&self) -> &Arc<SessionService> {
		&self.session_service
	}

	type Services = NativeServices;
	fn services(&self) -> &Self::Services {
		&self.services
	}
}

/// [Review]
/// 4 different spawn methods exist in the runtime namespace of the app.
/// Each of them was added at some time for some reason.
///
/// The code compiles on all targetted platforms, web, server, native.
///
///
/// 1: NativeExecutor inherent method
/// 2: NativeExecutor trait tmplementation.
/// 3: NativeRuntime runtime trait implementation
/// 3: NativeRuntime executor trait implementation

pub struct NativeAppContext<'a> {
	pub base: AppContext<'a, NativeRuntime, NativeExecutor>,
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

// impl SendSpawnRuntime for NativeRuntime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + Send + 'static,
// 	{
// 		self.handle.spawn(future);
// 	}
// }

// impl SpawnRuntime for NativeRuntime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + std::marker::Send + 'static,
// 	{
// 		self.handle.spawn(future);
// 	}
// }

// impl EventReceiver for async_broadcast::Receiver<e::Event> {
// 	fn try_recv(&mut self) -> Option<e::Event> {
// 		self.try_recv().ok()
// 	}
// }

// #[derive(Clone)]
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
