// use crate::prelude::*;
// // https://github.com/rust-lang/rust/issues/41517
// // https://github.com/rust-lang/rust/issues/55628
// // https://github.com/rust-lang/rfcs/pull/1733

// pub trait Clock {
// 	fn now(&self) -> String;

// 	/// Run once and return.
// 	fn run_once(&self);

// 	/// Run repeatedly in the foreground.
// 	fn run_foreground(&self, interval: Duration);

// 	/// Spawn the clock as a background process/task.
// 	fn run_background(&self, interval: Duration);
// }

// fn time_now() -> String {
// 	use chrono::Utc;
// 	let now = Utc::now();
// 	let format = "%B %-d, %Y at %-I:%M:%S %p UTC";
// 	now.format(format).to_string()
// }
// pub struct HostClock;

// impl Clock for HostClock {
// 	fn now(&self) -> String {
// 		#[cfg(target_arch = "wasm32")]
// 		{
// 			// Replace with JS/browser time later.
// 			0
// 		}

// 		#[cfg(not(target_arch = "wasm32"))]
// 		{
// 			time_now()
// 		}
// 	}

// 	fn run_once(&self) {
// 		println!("tick: {}", self.now());
// 	}

// 	fn run_foreground(&self, interval: Duration) {
// 		loop {
// 			self.run_once();
// 			std::thread::sleep(interval);
// 		}
// 	}

// 	fn run_background(&self, interval: Duration) {
// 		#[cfg(not(target_arch = "wasm32"))]
// 		{
// 			std::thread::spawn(move || {
// 				let clock = HostClock;

// 				loop {
// 					clock.run_once();
// 					std::thread::sleep(interval);
// 				}
// 			});
// 		}

// 		#[cfg(target_arch = "wasm32")]
// 		{
// 			// Later: window.setInterval(...) / async task.
// 			let _ = interval;
// 			println!("WASM background clock");
// 		}
// 	}
// }


fn main(){}
