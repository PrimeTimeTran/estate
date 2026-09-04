```rust
fn spawn_clock(&mut self, proxy: EventLoopProxy<AppEvent>) {
	// =========================================================================
	// NATIVE APP CLOCK
	// =========================================================================
	//
	// This function creates ONE long-lived asynchronous task.
	//
	// Important distinction:
	//
	//     spawn() is called ONCE
	//     the future created by async move { ... } is then POLLED repeatedly
	//     each time it yields at an `.await`.
	//
	// So the executor is not "called every tick". The executor schedules the
	// clock task; the clock task itself produces the individual ticks.
	//
	// Conceptually:
	//
	//     NativeApp
	//        │
	//        └── Arc<NativeRuntime>
	//                │
	//                └── NativeExecutor
	//                        │
	//                        └── spawn(clock future)
	//                                  │
	//                                  ├── tick
	//                                  ├── await sleep
	//                                  ├── tick
	//                                  ├── await sleep
	//                                  └── ...
	//
	// This distinction between "starting a future" and "polling a future"
	// is fundamental to understanding async Rust.
	// =========================================================================

	println!("Spawn Clock Start");

	// =========================================================================
	// SHARED SHUTDOWN STATE
	// =========================================================================
	//
	// `is_clocking` is owned by NativeApp, but the spawned async task needs to
	// outlive this function call and therefore cannot simply borrow `self`.
	//
	// `Arc` gives shared ownership of the same allocation.
	//
	//     NativeApp ──────┐
	//                     │
	//                     ▼
	//                  Arc<AtomicBool>
	//                     ▲
	//                     │
	//              clock task
	//
	// `clone()` does NOT clone the AtomicBool itself. It clones the Arc,
	// increasing the reference count so both NativeApp and the async task
	// can safely own a reference to the same AtomicBool.
	//
	// The task uses `load()` on every loop iteration to determine whether
	// NativeApp has requested shutdown.
	// =========================================================================

	let running = Arc::clone(&self.is_clocking);

	// =========================================================================
	// RUNTIME HANDLE
	// =========================================================================
	//
	// `self.handle()` gives us the Tokio runtime handle.
	//
	// A `Handle` is essentially a way to access an already-running Tokio
	// runtime from somewhere that does not itself own the runtime.
	//
	// It is NOT the executor abstraction used below. It is the concrete Tokio
	// mechanism underneath the native runtime.
	//
	// We currently don't use this local variable directly because the clock
	// is intentionally going through our `Executor` abstraction instead.
	//
	// Keeping this here is useful while understanding the architecture:
	//
	//     NativeApp
	//         │
	//         ▼
	//     Tokio Handle
	//
	// versus:
	//
	//     NativeApp
	//         │
	//         ▼
	//     NativeRuntime
	//         │
	//         ▼
	//     NativeExecutor
	//         │
	//         ▼
	//     Tokio
	//
	// The second path is the abstraction we are exercising here.
	// =========================================================================

	let handle: Handle = self.handle();

	// =========================================================================
	// RUNTIME OWNERSHIP
	// =========================================================================
	//
	// `runtime_old()` returns an `Arc<NativeRuntime>`.
	//
	// This matters because the async task is going to capture `task_runtime`
	// with `async move`. A spawned task generally needs to be able to live
	// independently of the stack frame of `spawn_clock()`.
	//
	// We therefore give the task its own Arc reference to NativeRuntime.
	//
	// The type is explicit here:
	//
	//     task_runtime: Arc<NativeRuntime>
	//
	// Read that from the inside out:
	//
	//     NativeRuntime
	//         concrete native runtime type
	//
	//     Arc<NativeRuntime>
	//         shared ownership of a NativeRuntime
	//
	// `Arc` does not make NativeRuntime magically thread-safe. The underlying
	// type and everything accessed through it must still satisfy the relevant
	// Send/Sync requirements when crossing threads.
	// =========================================================================

	let task_runtime: Arc<NativeRuntime> = self.runtime_old().clone();

	// =========================================================================
	// MOVING FROM RUNTIME TO EXECUTOR
	// =========================================================================
	//
	// `NativeRuntime` contains an executor:
	//
	//     NativeRuntime
	//         └── executor: NativeExecutor
	//
	// We clone that executor so the spawned task can use it if necessary.
	//
	// The important type transition is:
	//
	//     Arc<NativeRuntime>
	//             │
	//             │ `.executor`
	//             ▼
	//     NativeExecutor
	//
	// These are THREE different types:
	//
	//     Arc<NativeRuntime>
	//     NativeRuntime
	//     NativeExecutor
	//
	// Rust does not automatically treat them as interchangeable just because
	// one contains another.
	// =========================================================================

	let executor: NativeExecutor = task_runtime.executor.clone();

	// =========================================================================
	// EXPLICIT TRAIT METHOD SELECTION
	// =========================================================================
	//
	// THIS LINE IS PARTICULARLY IMPORTANT:
	//
	//     <NativeExecutor as Executor>::spawn(...)
	//
	// It means:
	//
	//     "Use the `Executor` trait implementation for `NativeExecutor`."
	//
	// It is equivalent to explicitly telling Rust which implementation we
	// want instead of relying on normal method-call syntax.
	//
	// Why was this necessary?
	//
	// We had both:
	//
	//     impl NativeExecutor {
	//         fn spawn(...) { ... }
	//     }
	//
	// and:
	//
	//     impl Executor for NativeExecutor {
	//         fn spawn(...) { ... }
	//     }
	//
	// These are two different methods that happen to have the same name.
	//
	// Normal:
	//
	//     executor.spawn(...)
	//
	// can resolve to the inherent method on NativeExecutor.
	//
	// The explicit form:
	//
	//     <NativeExecutor as Executor>::spawn(&executor, ...)
	//
	// removes the ambiguity.
	//
	// Read the syntax as:
	//
	//     <ConcreteType as Trait>::method(...)
	//
	// or:
	//
	//     <NativeExecutor as Executor>::spawn(...)
	//
	// which means:
	//
	//     "For the type NativeExecutor, use its implementation of Executor."
	//
	// This is one of the most useful pieces of syntax to understand when
	// debugging Rust's trait system.
	// =========================================================================

	<NativeExecutor as Executor>::spawn(&executor, async move {

		// =====================================================================
		// THE EXECUTOR HAS NOW ACCEPTED THE FUTURE
		// =====================================================================
		//
		// This println happens when Tokio actually begins polling the async
		// task. It is NOT the same thing as the call to `Executor::spawn()`.
		//
		// The lifecycle is roughly:
		//
		//     Executor::spawn(...)
		//             │
		//             ▼
		//     future is scheduled
		//             │
		//             ▼
		//     Tokio eventually polls future
		//             │
		//             ▼
		//     async block starts executing
		//
		// This distinction is why async Rust can initially feel strange:
		// creating/scheduling a future and executing its body are separate
		// concepts.
		// =====================================================================

		println!("Handle triggered executor");

		// =====================================================================
		// CLOCK STATE
		// =====================================================================
		//
		// Everything below lives INSIDE the spawned future.
		//
		// `views`, `current_time`, and `view_index` therefore belong to this
		// clock task. They persist across `.await` points because the future
		// stores its state between polls.
		// =====================================================================

		let views = [
			ViewType::ProblemScreen,
			ViewType::DashboardScreen,
			ViewType::MarkdownView,
			ViewType::ProblemScreen,
			ViewType::WaterfallScreen,
			ViewType::ProblemScreen,
			ViewType::TaskManagerScreen,
			ViewType::ProblemsScreen,
		];

		let mut current_time = 10;
		let mut view_index = 0;

		// =====================================================================
		// THE CLOCK LOOP
		// =====================================================================
		//
		// This is where the repeated "ticks" actually happen.
		//
		// Notice that Executor::spawn() is NOT called again.
		//
		// There is one spawned future containing one loop:
		//
		//     spawn ──► future
		//                  │
		//                  ▼
		//                loop
		//                  │
		//                  ▼
		//                tick
		//                  │
		//                  ▼
		//              sleep().await
		//                  │
		//                  ▼
		//             future yields
		//                  │
		//                  ▼
		//              Tokio polls
		//                  │
		//                  ▼
		//                tick
		//                  │
		//                 ...
		//
		// The `.await` is what gives Tokio the opportunity to run other tasks.
		// =====================================================================

		while running.load(Ordering::Relaxed) {

			println!("Native App Tick");

			// -----------------------------------------------------------------
			// Notify the winit event loop.
			//
			// The async task is running independently of the winit event loop.
			// `EventLoopProxy` provides a way for this background task to wake
			// the winit event loop and submit an AppEvent to it.
			// -----------------------------------------------------------------

			let _ = proxy.send_event(
				AppEvent::TickClock(format!(" {}s", current_time))
			);

			tracing::info!("NativeApp clock {}", current_time);

			// -----------------------------------------------------------------
			// Every ten seconds, rotate to another view.
			// -----------------------------------------------------------------

			if current_time == 0 {
				current_time = 10;

				view_index = (view_index + 1) % views.len();
				let view = views[view_index];

				tracing::info!(
					"⏩ Native App Clock navigation → {:?}",
					view
				);

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

				task_runtime.emit(
					e::Event::app(
						e::Klass::Navigate(view)
					)
				);

				let _ = proxy.send_event(AppEvent::RuntimeEvent);

			} else {
				current_time -= 1;
			}

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

			task_runtime
				.sleep(std::time::Duration::from_secs(1))
				.await;
		}

		// =====================================================================
		// SHUTDOWN
		// =====================================================================
		//
		// When NativeApp sets `is_clocking` to false, the next loop condition:
		//
		//     running.load(...)
		//
		// becomes false and the future completes.
		//
		// At that point Tokio considers this task finished.
		// =====================================================================
	});
}
```
