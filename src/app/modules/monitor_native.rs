use std::{ path::{ Path } };
use notify::{ Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher };

use crate::app::monitor::{ Monitor };

#[derive(Debug)]
pub struct StateMonitor {
	watcher: notify::RecommendedWatcher,
	rx: tokio::sync::mpsc::Receiver<()>,
}

impl StateMonitor {
	pub fn new(path: &Path) -> anyhow::Result<Self> {
		let (tx, rx) = tokio::sync::mpsc::channel(1);

		let mut watcher = RecommendedWatcher::new(move |result: Result<Event, notify::Error>| {
			let Ok(event) = result else {
				return;
			};

			if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
				let _ = tx.blocking_send(());
			}
		}, Config::default())?;

		if let Some(parent) = path.parent() {
			watcher.watch(parent, RecursiveMode::NonRecursive)?;
		}

		Ok(Self { watcher, rx })
	}

	pub fn poll(&mut self) -> bool {
		self.rx.try_recv().is_ok()
	}
}

#[derive(Debug)]
pub struct NativeMonitor {
	watcher: notify::RecommendedWatcher,
	rx: tokio::sync::mpsc::Receiver<()>,
}

impl NativeMonitor {
	pub fn new() -> anyhow::Result<Self> {
		use notify::{ Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher };
		let (tx, rx) = tokio::sync::mpsc::channel(1);
		let mut watcher = RecommendedWatcher::new(move |result: Result<Event, notify::Error>| {
			let Ok(event) = result else {
				return;
			};
			if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
				let _ = tx.blocking_send(());
			}
		}, Config::default())?;

		Ok(Self {
			watcher,
			rx,
		})
	}
}

impl Monitor for NativeMonitor {
	fn watch(&mut self) {}
	fn rx(&mut self) {}
	fn poll(&mut self) -> bool {
		self.rx.try_recv().is_ok()
	}
}

// pub(crate) mod monitor_native;
