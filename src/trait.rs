use crate::{e, prelude::*};

// [Potential Renames]
// App, AppPlatform, Host, AppHost, Engine, CoreEngine, AppContext, Environment
pub trait Runtime: Clone + Sync + std::marker::Send + 'static {
	// type Api: Api;
	// fn api(&self) -> &Self::Api;

	// Services own long-lived responsibilities and their concurrency/lifecycle;
	//
	// Events are the standardized mechanism by which those services expose meaningful
	// changes to the rest of the application; the Runtime owns the services and EventBus,
	// while the Dispatcher routes those events to consumers.
	type EventReceiver: EventReceiver;
	fn emit(&self, event: e::Event);
	fn event_processed(&self);
	fn subscribe(&self) -> Self::EventReceiver;
	fn try_recv(&self) -> Option<e::Event>;
	fn start_dispatcher(self: &Arc<Self>);
	fn state(&self) -> &RuntimeState;
	fn save(&self, state: &EstateState) -> Result<()>;
	fn session(&self) -> Session;

	async fn sleep(&self, duration: std::time::Duration);
	fn tasks(&self) -> &Arc<RwLock<TaskManager>>;

	fn state_service(&self) -> &Arc<StateService>;
	fn session_service(&self) -> &Arc<SessionService>;

	// fn spawn<F>(&self, future: F)
	// where
	// 	F: std::future::Future<Output = ()> + Send + 'static;
}
pub trait Spawner: Clone + 'static {
	fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + 'static;
}

pub trait Executor: Clone + 'static {
	fn spawn(&self, future: impl Future<Output = ()> + 'static);
}

// pub trait SendSpawnRuntime: Runtime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + Send + 'static;
// }
// pub trait SpawnRuntime: Runtime {
// 	fn spawn<F>(&self, future: F)
// 	where
// 		F: Future<Output = ()> + 'static;
// }

#[async_trait::async_trait]
pub trait EventHandler<R: Runtime>: 'static {
	async fn handle(&self, event: &e::Event, runtime: &R);
}

pub trait EventReceiver {
	fn try_recv(&mut self) -> Option<e::Event>;
}
