use crate::doc;
use crate::{app::prelude::*, r#trait::Context};

use crate::{
	AppEvent, DaemonCommand, NativeExecutor,
	api::NativeApiClient,
	app::{Runtime, model::EstateEngine},
	e,
	native::router,
	spawn_global_cursor_daemon,
};

use tokio::runtime::Handle;
use tray_icon::{
	TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
};
use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
	window::WindowId,
};

// macro_rules! doc {
// 	($text:expr) => {{ $text }};
// }

pub struct NativeApp {
	pub app: AppRuntime<NativeRuntime, NativeExecutor>,
	pub host: NativeHost,
	pub runtime: NativeRuntime,
	// Receiver channel for process/daemon
	pub daemon_rx: Option<mpsc::Receiver<DaemonCommand>>,
	// Sender channel for process/daemon
	pub daemon_tx: mpsc::Sender<DaemonCommand>,
	pub hotkey_manager: GlobalHotkeys,
	pub is_clocking: Arc<AtomicBool>,
	pub menu: Option<TrayMenu>,
	pub menu_bar: Option<Menu>,
	pub monitor: NativeMonitor,
	pub tokio: tokio::runtime::Runtime,
	pub tray_clock: Option<TrayIcon>,
	pub tray_cursor: Option<TrayIcon>,
	pub windows: Vec<AppWindow>,
}

impl Context for NativeApp {
	type Host = NativeHost;
	type Runtime = NativeRuntime;
	type Args = Cli;
	fn new() -> Result<Self> {
		NativeApp::new()
	}
	fn host(&self) -> &Self::Host {
		&self.host
	}
	fn runtime(&self) -> &Self::Runtime {
		&self.runtime
	}
	fn run(&mut self, cli: Self::Args) -> Result<()> {
		NativeApp::run(self, cli)
	}
	fn foo(&self, args: String) -> Result<()> {
		NativeApp::foo(&self, args)
	}
	fn bar(&self, args: String) -> Result<()> {
		NativeApp::bar(&self, args)
	}
}

impl NativeApp {
	pub fn new() -> Result<Self> {
		let tokio = tokio::runtime::Runtime::new()?;
		let handle = tokio.handle().clone();
		// Runtime owns all runtime infrastructure:
		// services, executor, state, event bus, session, etc.
		let runtime = tokio.block_on(NativeRuntime::new(handle.clone()))?;

		// Engine owns the domain/application engine and uses Runtime.
		let engine = EstateEngine::new(runtime.clone())?;

		let app = AppRuntime::new(engine.clone(), runtime.executor.clone());
		app.start();
		app.start_services();

		let host = NativeHost::new();
		let (daemon_tx, daemon_rx) = mpsc::channel(100);
		Ok(Self {
			app,
			host,
			runtime,
			is_clocking: Arc::new(AtomicBool::new(true)),
			daemon_rx: Some(daemon_rx),
			daemon_tx,
			hotkey_manager: GlobalHotkeys::new().unwrap(),
			menu: None,
			menu_bar: None,
			monitor: NativeMonitor::new()?,
			tokio,
			tray_clock: None,
			tray_cursor: None,
			windows: vec![],
		})
	}
	fn foo(&self, cli: String) -> Result<()> {
		todo!("run")
	}
	fn bar(&self, cli: String) -> Result<()> {
		todo!("run")
	}
	// Inner NativeApp
	fn runtime_old(&self) -> Arc<NativeRuntime> {
		// [Flexibility]
		// Decide later if theres any bad things that can happen from enabling app runtime
		// access.
		Arc::clone(&self.app.engine.runtime)
	}
	pub fn handle(&self) -> tokio::runtime::Handle {
		self.tokio.handle().clone()
	}
}
impl NativeApp {
	pub fn run(&mut self, cli: Cli) -> Result<()> {
		tracing::debug!(">>> NativeApp::run entered");
		let result = match cli.command {
			None | Some(Command::Start { .. }) | Some(Command::Tray) => self.start_runtime(),
			Some(_) => self.tokio.block_on(async {
				let ctx = cli::context::Context::new();
				router::execute(cli, ctx, self.app.engine.clone()).await
			}),
		};
		tracing::debug!(">>> NativeApp::run returning");
		result
	}
	fn start_runtime(&mut self) -> Result<()> {
		// // App
		// self.engine.runtime.spawn(future);       // ✅
		// // NativeRuntime
		// self.handle.spawn(future);               // ✅
		// // NativeApp
		// self.tokio = Runtime::new()?;             // ✅
		// // App
		// tokio::spawn(future);                     // ❌
		// // App
		// tokio::runtime::Handle::current();        // ❌
		// // AppContext
		// tokio::runtime::Handle;
		// 1. Create Tokio first.
		// AppRuntime
		self.runtime.start_services();
		// EstateEngineRuntime
		self.runtime().start_services();
		let daemon_rx = self.daemon_rx.take().expect("daemon already started");
		let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel::<Result<Arc<NativeApiClient>>>(1);
		self.spawn_daemon(daemon_rx, ready_tx);
		self.spawn_global_hotkey_daemon()?;
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.with_activation_policy(ActivationPolicy::Regular)
			.build()?;
		if let Some(menu) = &self.menu_bar {
			menu.init_for_nsapp();
		}
		let proxy = event_loop.create_proxy();
		self.spawn_clock(proxy.clone());
		self.spawn_cursor_daemon(proxy.clone());
		self.spawn_signal_handler(proxy.clone());
		self.runtime_old().attach_event_proxy(proxy);
		self
			.runtime_old()
			.emit(e::Event::app(e::Klass::SessionStart));
		event_loop.run_app(self)?;
		tracing::info!(">>> NativeApp::start_runtime returning");
		Ok(())
	}
	fn spawn_clock(&mut self, proxy: EventLoopProxy<AppEvent>) {
		doc!(
			r#"
        =========================================================================
    		NATIVE APP CLOCK
    		=========================================================================

    		This function creates ONE long-lived asynchronous task.

    		Important distinction:

    		    spawn() is called ONCE
    		    the future created by async move { ... } is then POLLED repeatedly
    		    each time it yields at an `.await`.

    		So the executor is not "called every tick". The executor schedules the
    		clock task; the clock task itself produces the individual ticks.

    		Conceptually:

    		    NativeApp
    		       │
    		       └── Arc<NativeRuntime>
    		               │
    		               └── NativeExecutor
    		                       │
    		                       └── spawn(clock future)
    		                                 │
    		                                 ├── tick
    		                                 ├── await sleep
    		                                 ├── tick
    		                                 ├── await sleep
    		                                 └── ...

    		This distinction between "starting a future" and "polling a future"
    		is fundamental to understanding async Rust.
    		=========================================================================
      "#
		);
		println!("Spawn Clock Start");
		doc!(
			r#"
        =========================================================================
    		SHARED SHUTDOWN STATE
    		=========================================================================
    		`is_clocking` is owned by NativeApp, but the spawned async task needs to
    		outlive this function call and therefore cannot simply borrow `self`.

    		`Arc` gives shared ownership of the same allocation.

    		    NativeApp ──────┐
    		                    │
    		                    ▼
   		                 Arc<AtomicBool>
    		                    ▲
    		                    │
   		             clock task

    		`clone()` does NOT clone the AtomicBool itself. It clones the Arc,
    		increasing the reference count so both NativeApp and the async task
    		can safely own a reference to the same AtomicBool.

    		The task uses `load()` on every loop iteration to determine whether
    		NativeApp has requested shutdown.
    		=========================================================================
		    "#
		);
		let running = Arc::clone(&self.is_clocking);
		doc!(
			r#"
    		=========================================================================
    		RUNTIME HANDLE
    		=========================================================================

    		`self.handle()` gives us the Tokio runtime handle.

    		A `Handle` is essentially a way to access an already-running Tokio
    		runtime from somewhere that does not itself own the runtime.

    		It is NOT the executor abstraction used below. It is the concrete Tokio
    		mechanism underneath the native runtime.

    		We currently don't use this local variable directly because the clock
    		is intentionally going through our `Executor` abstraction instead.

    		Keeping this here is useful while understanding the architecture:

    		    NativeApp
    		        │
    		        ▼
    		    Tokio Handle

    		versus:

    		    NativeApp
    		        │
    		        ▼
    		    NativeRuntime
    		        │
    		        ▼
    		    NativeExecutor
    		        │
    		        ▼
    		    Tokio

    		The second path is the abstraction we are exercising here.
    		=========================================================================
      "#
		);
		let handle: Handle = self.handle();
		doc!(
			r#"
    		=========================================================================
    		RUNTIME OWNERSHIP
    		=========================================================================

    		`runtime_old()` returns an `Arc<NativeRuntime>`.

    		This matters because the async task is going to capture `task_runtime`
    		with `async move`. A spawned task generally needs to be able to live
    		independently of the stack frame of `spawn_clock()`.

    		We therefore give the task its own Arc reference to NativeRuntime.

    		The type is explicit here:

    		    task_runtime: Arc<NativeRuntime>

    		Read that from the inside out:

    		    NativeRuntime
    		        concrete native runtime type

    		    Arc<NativeRuntime>
    		        shared ownership of a NativeRuntime

    		`Arc` does not make NativeRuntime magically thread-safe. The underlying
    		type and everything accessed through it must still satisfy the relevant
    		Send/Sync requirements when crossing threads.
    		=========================================================================
      "#
		);
		let task_runtime: Arc<NativeRuntime> = self.runtime_old().clone();
		doc!(
			r#"
    		=========================================================================
    		MOVING FROM RUNTIME TO EXECUTOR
    		=========================================================================

    		`NativeRuntime` contains an executor:

    		    NativeRuntime
    		        └── executor: NativeExecutor

    		We clone that executor so the spawned task can use it if necessary.

    		The important type transition is:

    		    Arc<NativeRuntime>
    		            │
    		            │ `.executor`
    		            ▼
    		    NativeExecutor

    		These are THREE different types:

    		    Arc<NativeRuntime>
    		    NativeRuntime
    		    NativeExecutor

    		Rust does not automatically treat them as interchangeable just because
    		one contains another.
    		=========================================================================
      "#
		);
		let executor: NativeExecutor = task_runtime.executor.clone();
		doc!(
			r#"
        =========================================================================
       	EXPLICIT TRAIT METHOD SELECTION
       	=========================================================================

       	THIS LINE IS PARTICULARLY IMPORTANT:

       	    <NativeExecutor as Executor>::spawn(...)

       	It means:

       	    "Use the `Executor` trait implementation for `NativeExecutor`."

       	It is equivalent to explicitly telling Rust which implementation we
       	want instead of relying on normal method-call syntax.

       	Why was this necessary?

       	We had both:

       	    impl NativeExecutor {
       	        fn spawn(...) { ... }
       	    }

       	and:

       	    impl Executor for NativeExecutor {
       	        fn spawn(...) { ... }
       	    }

       	These are two different methods that happen to have the same name.

       	Normal:

       	    executor.spawn(...)

       	can resolve to the inherent method on NativeExecutor.

       	The explicit form:

       	    <NativeExecutor as Executor>::spawn(&executor, ...)

       	removes the ambiguity.

       	Read the syntax as:

       	    <ConcreteType as Trait>::method(...)

       	or:

       	    <NativeExecutor as Executor>::spawn(...)

       	which means:

       	    "For the type NativeExecutor, use its implementation of Executor."

       	This is one of the most useful pieces of syntax to understand when
       	debugging Rust's trait system.
       	=========================================================================
      "#
		);
		<NativeExecutor as Executor>::spawn(&executor, async move {
			doc!(
				r#"
  			  =====================================================================
     			THE EXECUTOR HAS NOW ACCEPTED THE FUTURE
     			=====================================================================

     			This println happens when Tokio actually begins polling the async
     			task. It is NOT the same thing as the call to `Executor::spawn()`.

     			The lifecycle is roughly:

     			    Executor::spawn(...)
     			            │
     			            ▼
     			    future is scheduled
     			            │
     			            ▼
     			    Tokio eventually polls future
     			            │
     			            ▼
     			    async block starts executing

     			This distinction is why async Rust can initially feel strange:
     			creating/scheduling a future and executing its body are separate
     			concepts.
     			=====================================================================
        "#
			);
			doc!(
				r#"
          =====================================================================
      		CLOCK STATE
      		=====================================================================

      		Everything below lives INSIDE the spawned future.

      		`views`, `current_time`, and `view_index` therefore belong to this
      		clock task. They persist across `.await` points because the future
      		stores its state between polls.
      		=====================================================================
        "#
			);
			let mut current_time = 10;
			let mut view_index = 0;
			doc!(
				r#"
  				=====================================================================
      		THE CLOCK LOOP
      		=====================================================================

      		This is where the repeated "ticks" actually happen.

      		Notice that Executor::spawn() is NOT called again.

      		There is one spawned future containing one loop:

      		    spawn ──► future
      		                 │
      		                 ▼
      		               loop
      		                 │
      		                 ▼
      		               tick
      		                 │
      		                 ▼
      		             sleep().await
      		                 │
      		                 ▼
      		            future yields
      		                 │
      		                 ▼
      		             Tokio polls
      		                 │
      		                 ▼
      		               tick
      		                 │
      		                ...

      		The `.await` is what gives Tokio the opportunity to run other tasks.
      		=====================================================================
        "#
			);
			while running.load(Ordering::Relaxed) {
				// -----------------------------------------------------------------
				// Notify the winit event loop.

				// The async task is running independently of the winit event loop.
				// `EventLoopProxy` provides a way for this background task to wake
				// the winit event loop and submit an AppEvent to it.
				// -----------------------------------------------------------------
				let _ = proxy.send_event(AppEvent::TickClock(format!(" {}s", current_time)));
				tracing::info!("NativeApp clock {}", current_time);
				// -----------------------------------------------------------------
				// Every ten seconds, rotate to another view.
				// -----------------------------------------------------------------
				if current_time == 0 {
					current_time = 10;
					view_index = (view_index + 1) % TICK_ITEMS_LENGTH;
					let view = TICK_ITEMS[view_index];
					tracing::info!("⏩ Native App Clock navigation → {:?}", view);
					doc!(
						r#"
							// -------------------------------------------------------------
							// `task_runtime` is an Arc<NativeRuntime>.
							//
							// Method lookup can automatically dereference the Arc and find
							// methods implemented on NativeRuntime.
							//
							// So:
							//
							//     task_runtime.emit(...)
							//
							// is conceptually:
							//
							//     NativeRuntime::emit(...)
							//
							// through the Arc.
							//
							// This is different from the earlier `executor.spawn(...)`
							// problem because here we're deliberately calling a method
							// belonging to the runtime object.
							// -------------------------------------------------------------
						"#
					);
					task_runtime.emit(e::Event::app(e::Klass::Navigate(view)));
					let _ = proxy.send_event(AppEvent::RuntimeEvent);
				} else {
					current_time -= 1;
				}
				doc!(
					r#"
        		// =================================================================
        		// ASYNC YIELD POINT
        		// =================================================================
        		//
        		// THIS is what turns the loop into an asynchronous clock.
        		//
        		// `sleep(...).await` does not block the Tokio worker thread.
        		//
        		// Instead:
        		//
        		//     clock task
        		//         │
        		//         ▼
        		//     sleep future
        		//         │
        		//         ▼
        		//     `.await`
        		//         │
        		//         ▼
        		//     task yields to Tokio
        		//
        		// Tokio can now execute other tasks while this clock is waiting.
        		//
        		// When the timer expires, Tokio wakes the task and eventually polls
        		// this future again. Execution resumes after `.await`.
        		//
        		// This is why the local variables above survive across the sleep:
        		// they are part of the state stored inside the future.
        		//
        		// Contrast this with:
        		//
        		//     std::thread::sleep(...)
        		//
        		// which would block the actual OS thread running the Tokio worker.
        		// =================================================================
          "#
				);
				task_runtime.sleep(std::time::Duration::from_secs(1)).await;
			}
			doc!(
				r#"
          =====================================================================
      		SHUTDOWN
      		=====================================================================

      		When NativeApp sets `is_clocking` to false, the next loop condition:

      		    running.load(...)

      		becomes false and the future completes.

      		At that point Tokio considers this task finished.
      		=====================================================================
        "#
			);
		});
	}
	fn spawn_cursor_daemon(&mut self, proxy: EventLoopProxy<AppEvent>) {
		spawn_global_cursor_daemon(proxy)
	}
	fn spawn_global_hotkey_daemon(&mut self) -> Result<()> {
		self.hotkey_manager.start();
		Ok(())
	}
	fn spawn_daemon(
		&mut self,
		mut rx: mpsc::Receiver<DaemonCommand>,
		ready_tx: std::sync::mpsc::SyncSender<Result<Arc<NativeApiClient>>>,
	) {
		let runtime = self.runtime_old();
		self.handle().spawn(async move {
			runtime.start_dispatcher();
			let daemon: Daemon<NativeRuntime> = Daemon::new(runtime.clone());
			let shutdown_token = daemon.shutdown_token.clone();
			let daemon_task = tokio::spawn(async move {
				let mut daemon = daemon;
				daemon.run_foreground().await
			});
			match rx.recv().await {
				Some(DaemonCommand::Stop) => {
					tracing::info!("daemon stop requested");
					shutdown_token.cancel();
					match daemon_task.await {
						Ok(Ok(())) => {
							tracing::info!("daemon stopped cleanly");
						}
						Ok(Err(error)) => {
							tracing::error!(%error, "daemon exited with error");
						}
						Err(error) => {
							tracing::error!(%error, "daemon task panicked");
						}
					}
				}
				None => {
					tracing::info!("daemon command channel closed");
					shutdown_token.cancel();
					let _ = daemon_task.await;
				}
			}
		});
	}
	fn spawn_signal_handler(&mut self, proxy: EventLoopProxy<AppEvent>) {
		std::thread::spawn(move || {
			tracing::info!("SIGNAL: thread started");
			let mut signals = Signals::new([SIGINT]).expect("failed to register SIGINT");
			tracing::info!("SIGNAL: waiting");
			if signals.forever().next().is_some() {
				tracing::info!("SIGNAL: received");
				let _ = proxy.send_event(AppEvent::Shutdown);
				tracing::info!("SIGNAL: event sent");
			}
			tracing::info!("SIGNAL: thread exiting");
		});
	}
	fn shutdown(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!(">>> shutting down runtime");
		let snapshot = {
			let runtime = self.runtime();
			let mut state = runtime.state.write();
			state.session.end();
			state.clone()
		};
		self.runtime().save(&snapshot);
		self.is_clocking.store(false, Ordering::Relaxed);
		self.hotkey_manager.shutdown();
		match self.daemon_tx.try_send(DaemonCommand::Stop) {
			Ok(()) => tracing::info!(">>> daemon stop sent"),
			Err(error) => tracing::error!(%error, ">>> daemon stop failed"),
		}
		tracing::info!(">>> runtime shutdown complete");
		event_loop.exit();
	}
}

fn fun_name1() -> &'static str {
	r#"
          -------------------------------------------------------------
					`task_runtime` is an Arc<NativeRuntime>.

					Method lookup can automatically dereference the Arc and find
					methods implemented on NativeRuntime.

					So:

					    task_runtime.emit(...)

					is conceptually:

					    NativeRuntime::emit(...)

					through the Arc.

					This is different from the earlier `executor.spawn(...)`
					problem because here we're deliberately calling a method
					belonging to the runtime object.
					-------------------------------------------------------------
"#
}

fn fun_name() {
	// =================================================================
	// ASYNC YIELD POINT
	// =================================================================
	//
	// THIS is what turns the loop into an asynchronous clock.
	//
	// `sleep(...).await` does not block the Tokio worker thread.
	//
	// Instead:
	//
	//     clock task
	//         │
	//         ▼
	//     sleep future
	//         │
	//         ▼
	//     `.await`
	//         │
	//         ▼
	//     task yields to Tokio
	//
	// Tokio can now execute other tasks while this clock is waiting.
	//
	// When the timer expires, Tokio wakes the task and eventually polls
	// this future again. Execution resumes after `.await`.
	//
	// This is why the local variables above survive across the sleep:
	// they are part of the state stored inside the future.
	//
	// Contrast this with:
	//
	//     std::thread::sleep(...)
	//
	// which would block the actual OS thread running the Tokio worker.
	// =================================================================
}
impl NativeApp {
	fn bootstrap() -> Result<(TrayMenu, TrayIcon)> {
		bootstrap()
	}
	fn window_by_type(&mut self, kind: WindowType) -> Option<&mut AppWindow> {
		self.windows.iter_mut().find(|window| window.kind == kind)
	}
	fn handle_event(&mut self, event: MenuEvent, event_loop: &ActiveEventLoop) {
		let Some(menu) = self.menu.as_ref() else {
			return;
		};
		let id = event.id();
		if id == menu.quit.id() {
			tracing::debug!(">>> tray quit requested");
			self.shutdown(event_loop);
			tracing::debug!(">>> event_loop.exit() called");
		} else if id == menu.dev.id() {
			self.open_window(event_loop, WindowType::DashboardScreen);
		} else if id == menu.oracle.id() {
			self.open_window(event_loop, WindowType::OracleScreen);
		} else if id == menu.task_manager.id() {
			self.open_window(event_loop, WindowType::TaskManagerScreen);
		} else if id == menu.new_task.id() {
			self.new_task();
		} else if id == menu.list_tasks.id() {
			self.show_tasks();
		} else if id == menu.clear_tasks.id() {
			self.clear_tasks();
		} else if id == menu.problem_screen.id() {
			tracing::info!("🧭 Menu → ProblemsScreen");
			self
				.runtime()
				.emit(e::Event::app(e::Klass::Navigate(ViewType::ProblemsScreen)));
			self.open_window(event_loop, WindowType::ProblemsScreen);
		}
	}
	fn open_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowType) {
		tracing::info!(" open window start");
		if self.window_by_type(kind).is_some() {
			return;
		}
		match Window::new(event_loop, self.app.view()) {
			Ok(window) => {
				tracing::info!(" open window end, new window");
				window.instance.set_title(self.app.view().name().into());
				self.windows.push(AppWindow {
					kind,
					view: self.app.view(),
					window,
				});
			}
			Err(error) => {
				tracing::error!("failed to create window: {error}");
			}
		}
	}
}
impl NativeApp {
	fn new_task(&mut self) {
		self.runtime().emit(e::Event::app(e::Klass::TaskRequested {
			request: TaskRequest::Create(TaskKind::SyncBookmarks),
		}));
	}
	fn show_tasks(&mut self) {
		self
			.runtime()
			.emit(e::Event::app(e::Klass::CommandExecuted {
				command: "task_list".into(),
			}));
	}
	fn clear_tasks(&mut self) {
		self
			.runtime()
			.emit(e::Event::app(e::Klass::CommandExecuted {
				command: "task_clear".into(),
			}));
	}
	#[tracing::instrument(
		target = "estate::discovery",
		name = "scan_workspace",
		skip(self),
		fields(flow_id = %Uuid::new_v4())
	)]
	async fn _scan_workspace(&mut self, path: &Path) -> Result<()> {
		tracing::info!("starting workspace scan");
		self._discover(path).await?;
		tracing::debug!("discovery complete");
		self._analyze().await?;
		tracing::debug!("analysis complete");
		self._build_graph().await?;
		tracing::info!("workspace scan complete");
		Ok(())
	}
	#[tracing::instrument(target = "estate::discovery", skip(self, path))]
	async fn _discover(&mut self, path: &Path) -> Result<()> {
		tracing::debug!(path = %path.display(), "discovering workspace");
		Ok(())
	}
	#[tracing::instrument(target = "estate::analysis", skip(self))]
	async fn _analyze(&mut self) -> Result<()> {
		tracing::debug!("analyzing workspace");
		Ok(())
	}
	#[tracing::instrument(target = "estate::graph", skip(self))]
	async fn _build_graph(&mut self) -> Result<()> {
		tracing::debug!("building semantic graph");
		Ok(())
	}
}
impl NativeApp {
	fn sync_views(&mut self) {
		for window in &mut self.windows {
			// println!("NativeApp {:?}", self.runtime.view);
			tracing::debug!("sync views {:?}", self.app.view.name());
			// window
			// 	.window
			// 	.sync_view(self.runtime.view, self.runtime.api.clone());
			window.window.instance.request_redraw();
			window.window.instance.set_title(self.app.view.name());
		}
	}
	fn set_menu_bar(&mut self, new_menu: Menu) {
		self.menu_bar = Some(new_menu);
		tracing::info!("setting menu bar");
		if let Some(menu) = &self.menu_bar {
			menu.init_for_nsapp();
		}
	}
	fn menu_bar(has_document: bool) -> Menu {
		let menu = Menu::new();
		menu.append(&Self::file_menu(has_document)).unwrap();
		menu.append(&Self::edit_menu()).unwrap();
		menu
	}
	fn file_menu(has_document: bool) -> Submenu {
		let menu = Submenu::new("File", true);
		menu.append(&MenuItem::new("New", true, None));
		menu.append(&MenuItem::new("Open…", true, None));
		menu.append(&PredefinedMenuItem::separator());
		menu.append(&MenuItem::new("Close", has_document, None));
		menu
	}
	fn edit_menu() -> Submenu {
		let menu = Submenu::new("Edit", true);
		menu.append(&PredefinedMenuItem::undo(None));
		menu.append(&PredefinedMenuItem::redo(None));
		menu.append(&PredefinedMenuItem::separator());
		menu.append(&PredefinedMenuItem::cut(None));
		menu.append(&PredefinedMenuItem::copy(None));
		menu.append(&PredefinedMenuItem::paste(None));
		menu
	}
	fn create_menu() -> Menu {
		let menu = Menu::new();
		let file = Submenu::new("File", true);
		file.append_items(&[
			&MenuItem::new("New", true, None),
			&MenuItem::new("Open…", true, None),
			&PredefinedMenuItem::separator(),
			&MenuItem::new("Close", true, None),
		]);
		let edit = Submenu::new("Edit", true);
		edit.append_items(&[
			&PredefinedMenuItem::undo(None),
			&PredefinedMenuItem::redo(None),
			&PredefinedMenuItem::separator(),
			&PredefinedMenuItem::cut(None),
			&PredefinedMenuItem::copy(None),
			&PredefinedMenuItem::paste(None),
		]);
		let view = Submenu::new("View", true);
		view.append_items(&[
			&MenuItem::new("Toggle Sidebar", true, None),
			&MenuItem::new("Fullscreen", true, None),
		]);
		menu.append_items(&[&file, &edit, &view]);
		menu
	}
}
impl ApplicationHandler<AppEvent> for NativeApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if self.menu_bar.is_none() {
			let menu = Self::menu_bar(true);
			menu.init_for_nsapp();
			self.menu_bar = Some(menu);
		}
		if self.windows.is_empty() {
			self.open_window(event_loop, crate::START_WINDOW);
		}
		if self.tray_clock.is_none() {
			let (menu, tray) = match Self::bootstrap() {
				Ok(value) => value,
				Err(error) => {
					tracing::error!(%error, "failed to bootstrap tray");
					return;
				}
			};
			self.menu = Some(menu);
			self.tray_clock = Some(tray);
			tracing::debug!("🔥 main tray initialized");
		}
		if self.tray_cursor.is_none() {
			match TrayIconBuilder::new()
				.with_icon(scroll_tray_icon())
				.with_tooltip("Estate Scroll Controller")
				.build()
			{
				Ok(tray) => {
					self.tray_cursor = Some(tray);
					tracing::debug!("🔥 scroll tray initialized");
				}
				Err(error) => {
					tracing::error!(%error, "failed to create scroll tray");
				}
			}
		}
	}
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		self.app.update();
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			self.handle_event(event, event_loop);
		}
	}
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		let Some(window) = self
			.windows
			.iter_mut()
			.find(|window| window.window.instance.id() == window_id)
		else {
			return;
		};
		let response = window
			.window
			.gui_state
			.on_window_event(&window.window.instance, &event);
		if response.repaint {
			window.window.instance.request_redraw();
		}
		match event {
			WindowEvent::CloseRequested => {
				tracing::info!("🛑 Window close requested for id: {:?}", window_id);
				self
					.windows
					.retain(|window| window.window.instance.id() != window_id);
				return;
			}
			WindowEvent::RedrawRequested => {
				if window.window.occluded {
					return;
				}
				let menu = {
					let event_rx = self.app.engine.runtime().subscribe();
					let mut ctx = AppContext {
						app: &mut self.app,
						input: IOState::default(),
						event_rx,
						last_revision: 0,
					};

					if let Err(e) = window.window.draw(&mut ctx) {
						tracing::error!("DEV >>> draw failed: {e:#}");
					}
				};
			}
			WindowEvent::Focused(true) => {
				window.window.instance.request_redraw();
			}
			WindowEvent::Occluded(occluded) => {
				window.window.occluded = occluded;
				if !occluded {
					window.window.instance.request_redraw();
				}
			}
			WindowEvent::Resized(size) => {
				if size.width == 0 || size.height == 0 {
					return;
				}
				window.window.config.width = size.width;
				window.window.config.height = size.height;
				window
					.window
					.surface
					.configure(&window.window.device, &window.window.config);
				window.window.needs_resize = false;
				window.window.instance.request_redraw();
			}
			_ => {}
		}
	}
	fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
		match event {
			AppEvent::RuntimeEvent => {
				self.app.update();
				self.sync_views();
			}
			AppEvent::Navigate(view) => {
				self.runtime().emit(e::Event::app(e::Klass::Navigate(view)));
				self.app.update();
				self.sync_views();
			}
			AppEvent::Shutdown => {
				tracing::info!(">>> shutdown event received");
				self.shutdown(event_loop);

				tracing::info!(">>> event_loop.exit() called");
			}
			AppEvent::CursorPosition { x, y } => {
				// let text = format!("↖ {:.0}  {:.0}", x, y);
				// let text = format!("← {:.0}  {:.0}", x, y);
				// let text = format!("→ {:.0}  {:.0}", x, y);
				// let text = format!("↑ {:.0}  {:.0}", x, y);
				// let text = format!("● {:.0}, {:.0}", x, y);
				// let text = format!("◉ {:.0}, {:.0}", x, y);
				let text = format!("⌖ {:.0}, {:.0}", x, y);
				// let text = format!("🟢 {:.0}, {:.0}", x, y);
				// let text = format!("🔵 {:.0}, {:.0}", x, y);
				// let text = format!("🟡 {:.0}, {:.0}", x, y);
				// let text = format!("🔴 {:.0}, {:.0}", x, y);
				// let region = if x < 960.0 { "← LEFT" } else { "RIGHT →" };
				if let Some(tray) = &self.tray_cursor {
					let _ = tray.set_title(Some(text));
				}
			}
			AppEvent::TickClock(text) => {
				if let Some(tray) = &self.tray_clock {
					let _ = tray.set_title(Some(text));
				}
				self.sync_views();
			}
			AppEvent::ModifiersChanged {
				alt,
				command,
				ctrl,
				shift,
			} => {}
			_ => {}
		}
	}
}

#[derive(Default)]
pub struct NativeHost {
	window: NativeWindow,
	storage: NativeStorage,
	clock: NativeClock,
}

impl NativeHost {
	fn new() -> Self {
		Self::default()
	}
}
#[derive(Debug, Default, Clone)]
pub struct NativeWindow;
#[derive(Debug, Default, Clone)]
pub struct NativeStorage;
#[derive(Debug, Default, Clone)]
pub struct NativeClock;

impl Clock for NativeClock {
	fn now(&self) -> std::time::Instant {
		todo!("now")
	}
}

impl Host for NativeHost {
	type Window = NativeWindow;
	type Storage = NativeStorage;
	type Clock = NativeClock;
	fn window(&self) -> &Self::Window {
		&self.window
	}
	fn storage(&self) -> &Self::Storage {
		&self.storage
	}
	fn clock(&self) -> &Self::Clock {
		&self.clock
	}
}

#[derive(Debug, Clone)]
pub struct NativeServices {
	persistence: NativePersistence,
	network: NativeNetwork,
	clock: NativeClock,
	api: NativeApiClient,
}

impl NativeServices {
	pub async fn connect() -> anyhow::Result<Self> {
		let api = NativeApiClient::connect().await?;
		Ok(Self {
			persistence: NativePersistence::default(),
			network: NativeNetwork::default(),
			clock: NativeClock::default(),
			api,
		})
	}
}

impl Services for NativeServices {
	type Persistence = NativePersistence;
	type Network = NativeNetwork;
	type Clock = NativeClock;
	type Client = NativeApiClient;

	fn persistence(&self) -> &Self::Persistence {
		todo!("");
	}
	fn network(&self) -> &Self::Network {
		todo!("")
	}
	fn clock(&self) -> &Self::Clock {
		todo!("");
	}
	fn api(&self) -> &Self::Client {
		&self.api
	}
}

#[derive(Debug, Default, Clone)]
pub struct NativePersistence;

impl Persistence for NativePersistence {
	fn load(&self, key: &str) -> Result<Option<Vec<u8>>> {
		todo!("")
	}
	fn save(&self, key: &str, value: &[u8]) -> Result<()> {
		todo!("")
	}
}
#[derive(Debug, Default, Clone)]
pub struct NativeNetwork;

impl Network for NativeNetwork {
	fn is_available(&self) -> bool {
		todo!("")
	}
}

impl Drop for NativeApp {
	fn drop(&mut self) {
		tracing::info!("💀 NativeApp DROPPED");
	}
}
