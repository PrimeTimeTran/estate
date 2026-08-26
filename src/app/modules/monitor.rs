use std::{ env, fs, path::{ Path, PathBuf } };

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

#[cfg(not(target_arch = "wasm32"))]
pub use crate::app::monitor_native::NativeMonitor;

// #[cfg(not(target_arch = "wasm32"))]
// pub(crate) mod monitor_native;
// #[cfg(not(target_arch = "wasm32"))]
// pub use monitor_native::{ * };

// impl NativeMonitor {
// 	pub fn new() -> anyhow::Result<Self> {
// 		Ok(Self {})
// 	}
// 	pub fn load() -> anyhow::Result<Self> {
// 		Ok(Self {})
// 	}
// }

// #[derive(Debug)]
// pub struct StateMonitor {
// 	watcher: notify::RecommendedWatcher,
// 	rx: tokio::sync::mpsc::Receiver<()>,
// }

// impl StateMonitor {
// 	pub fn new(path: &Path) -> anyhow::Result<Self> {
// 		use notify::{ Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher };

// 		let (tx, rx) = tokio::sync::mpsc::channel(1);

// 		let mut watcher = RecommendedWatcher::new(move |result: Result<Event, notify::Error>| {
// 			let Ok(event) = result else {
// 				return;
// 			};

// 			if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
// 				let _ = tx.blocking_send(());
// 			}
// 		}, Config::default())?;

// 		if let Some(parent) = path.parent() {
// 			watcher.watch(parent, RecursiveMode::NonRecursive)?;
// 		}

// 		Ok(Self { watcher, rx })
// 	}

// 	pub fn poll(&mut self) -> bool {
// 		self.rx.try_recv().is_ok()
// 	}
// }
