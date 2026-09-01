use notify::{Event, EventKind};

use crate::app::*;

pub trait Monitor {
	fn watch(&mut self);
	fn rx(&mut self);
	fn poll(&mut self) -> bool;
}

#[cfg(target_arch = "wasm32")]
pub struct WebMonitor;

#[cfg(target_arch = "wasm32")]
impl Monitor for WebMonitor {
	fn watch(&mut self) {}

	fn rx(&mut self) {}

	fn poll(&mut self) -> bool {
		false
	}
}

/// Types
/// - FS Change (CRUD)- A filesystem watcher has an underlying blocking/event-driven mechanism:
/// 	runtime.emit(Event::FileChanged(...))
/// - Clock Running (Pomodoro) - tokio::interval(...)
/// 	runtime.emit(Event::PomodoroTick(...))
/// - Task Lifecycle (CRUD) - Task/Job lifecycle event stream. Also needs write access
/// 	runtime.emit(Event::Shortcut(...))
/// - Cursor Movement - OS event stream
/// - Shortcut Trigger - OS event stream
/// - Notification -
/// - Events - Analysis of Ownership, FS indexer, File Downloads
impl StateMonitor {
	pub fn with_file() {
		// let (sender, receiver) = channel::channel::<i32>(10);
	}
	// pub fn new(path: &Path) -> Result<Self> {
	// 	let (tx, rx) = mpsc::channel(1);

	// 	let mut watcher = RecommendedWatcher::new(
	// 		move |result: Result<Event, notify::Error>| {
	// 			let Ok(event) = result else {
	// 				return;
	// 			};

	// 			if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
	// 				// Channel size is 1 intentionally: we only care that
	// 				// something changed, not how many filesystem events
	// 				// occurred.
	// 				let _ = tx.blocking_send(());
	// 			}
	// 		},
	// 		Config::default(),
	// 	)?;

	// 	// Watch the parent directory rather than the file itself.
	// 	//
	// 	// Editors commonly save by writing a temporary file and then
	// 	// renaming it over the original file.
	// 	let watch_path = path.parent().unwrap_or(path);

	// 	watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

	// 	Ok(Self { watcher, rx })
	// }

	/// Returns true if the watched resource changed since the last poll.
	///
	/// Multiple filesystem events are collapsed into one logical change.
	pub fn poll(&mut self) -> bool {
		let mut changed = false;

		while self.rx.try_recv().is_ok() {
			changed = true;
		}

		changed
	}
}

impl Monitor for StateMonitor {
	fn watch(&mut self) {}
	fn rx(&mut self) {}
	fn poll(&mut self) -> bool {
		self.poll()
	}
}
#[derive(Debug)]
pub struct NativeMonitor {
	watcher: notify::RecommendedWatcher,
	rx: tokio::sync::mpsc::Receiver<()>,
}

impl NativeMonitor {
	pub fn new() -> Result<Self> {
		let (tx, rx) = tokio::sync::mpsc::channel(1);
		let mut watcher = RecommendedWatcher::new(
			move |result: Result<Event, notify::Error>| {
				let Ok(event) = result else {
					return;
				};
				if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
					let _ = tx.blocking_send(());
				}
			},
			Config::default(),
		)?;

		Ok(Self { watcher, rx })
	}
}

impl Monitor for NativeMonitor {
	fn watch(&mut self) {}
	fn rx(&mut self) {}
	fn poll(&mut self) -> bool {
		self.rx.try_recv().is_ok()
	}
}

#[derive(Debug)]
pub struct StateMonitor {
	watcher: RecommendedWatcher,
	rx: mpsc::Receiver<()>,
}

impl Default for StateMonitor {
	fn default() -> Self {
		let (tx, rx) = mpsc::channel(100);

		let watcher = RecommendedWatcher::new(
			move |_| {
				let _ = tx.try_send(());
			},
			notify::Config::default(),
		)
		.expect("failed to create state watcher");

		Self { watcher, rx }
	}
}

impl StateMonitor {
	pub fn new(path: &Path) -> notify::Result<Self> {
		let (tx, rx) = mpsc::channel(16);

		let mut watcher = RecommendedWatcher::new(
			move |result: notify::Result<Event>| {
				if result.is_ok() {
					let _ = tx.try_send(());
				}
			},
			Config::default(),
		)?;

		if let Some(parent) = path.parent() {
			watcher.watch(parent, RecursiveMode::NonRecursive)?;
		}

		Ok(Self { watcher, rx })
	}

	pub fn try_changed(&mut self) -> bool {
		self.rx.try_recv().is_ok()
	}
}
