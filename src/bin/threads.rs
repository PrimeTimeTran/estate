// https://www.youtube.com/watch?v=SbcYv5EQGNM&t=32s
// https://www.youtube.com/watch?v=rNhfQimiwEs
// https://www.youtube.com/watch?v=8gNKE5jVqYY
#![allow(warnings)]
// 1. Race Conditions
// Deadlocks
// Invalid Memory Access
use std::{
	sync::{
		Arc, Barrier, Mutex,
		atomic::{AtomicUsize, Ordering},
	},
	thread,
	time::Duration,
};
fn main() {
	// threads_are_async();
	// demo_loop();
	// threads_outlive_functions();
	// move_transfers_ownership();
	// arc_handles_ownership();
	// cross_thread_data_needs_mutex();
	mutex_race_depends_on_scheduling();
	mutex_race_is_deterministically_reproduced();
	println!("{:?} Main Thread done", thread::current().id());
}
fn threads_are_async() {
	let t2 = thread::spawn(|| {
		// This thread is not guaranteed to print
		// thread::sleep(Duration::from_millis(10));
		println!("{:?} Spawned thread '2' running...", thread::current().id());
		println!("{:?} thread '2' done", thread::current().id())
		// To ensure we see the spawned print, we must use the join handle returned from thread::spawn()
		// Otherwise the main thread may exit before the spawned thread finishes and we miss the print
		// outside of the closure
		// t.join().unwrap();
	});
	// t2.join().unwrap();
	println!("{:?} Spawned thread '2' done", thread::current().id());
}
fn demo_loop() {
	let t3 = thread::spawn(|| {
		for i in (1..3) {
			println!(
				"{:?} Spawned thread '3' running... {i}",
				thread::current().id()
			);
			thread::sleep(Duration::from_millis(10));
		}
		println!("{:?} thread '3' done", thread::current().id())
	});
	t3.join().unwrap();
	println!("{:?} Spawned thread '3' done", thread::current().id());
}
fn threads_outlive_functions() {
	// "Closure may outlive the current function"
	// The solution is to transfer ownership of any data it takes with the move
	let s = "hello";
	// let t4 = thread::spawn(|| {
	let t4 = thread::spawn(move || {
		println!(
			"{:?} Spawned thread '4' running... {}",
			thread::current().id(),
			s
		);
		println!("{:?} thread '4' done", thread::current().id())
	});
	t4.join().unwrap();
}
fn move_transfers_ownership() {
	// [Bug]: use of moved value: `s`
	// [from]: let s = use of moved value: `s`
	// String::from owns it's own data and transfers it
	// "s" owns owns a heap-allocated String.
	// let s = String::from("hello");
	//
	// [Solution]:  &str
	// &'static str is it's type
	// &str is a ref to string data and string literals are stored in the program's binary and live for the entire program.
	let s = "hello";
	let t5 = thread::spawn(move || {
		// move here means
		// "Move the variables captured by the closure into the closure."
		println!(
			"{:?} Spawned thread '5' running... {}",
			thread::current().id(),
			s
		);
	});
	let t6 = thread::spawn(move || {
		println!(
			"{:?} Spawned thread '6' running... {}",
			thread::current().id(),
			s
		);
	});
	t5.join().unwrap();
	t6.join().unwrap();
}
fn arc_handles_ownership() {
	// Arc (Atomically Reference Counted) lets multiple threads
	// own the same data at the same time.
	//
	// `s` owns the String, but cloning the Arc does NOT clone the
	// String itself. Instead, each Arc clone points to the same
	// allocation in memory and increments its atomic reference count.
	//
	// This means:
	//
	//     s  ──┐
	//     s1 ──┼──> "hello" (one String in memory)
	//     s2 ──┘
	//
	// Each thread can therefore take ownership of its own Arc handle
	// while still referring to the exact same underlying data.
	//
	// Arc is thread-safe because its reference count is atomic, so
	// multiple threads can clone/drop their Arc handles concurrently.
	//
	// Arc doesn't make multiple copies of the data; it gives multiple threads shared ownership of the same data.
	let s = Arc::new(String::from("hello"));
	let s1 = Arc::clone(&s);
	let s2 = Arc::clone(&s);
	let t5 = thread::spawn(move || {
		println!(
			"{:?} Spawned thread '7' running... {}",
			thread::current().id(),
			s1
		);
	});
	let t6 = thread::spawn(move || {
		println!(
			"{:?} Spawned thread '8' running... {}",
			thread::current().id(),
			s2
		);
	});
	t5.join().unwrap();
	t6.join().unwrap();
}
fn cross_thread_data_needs_mutex() {
	// Arc gives multiple threads shared ownership of the same data,
	// but Arc alone does NOT allow us to mutate the wrapped value.
	//
	// This is the problem we need Mutex to solve:
	//
	//     Arc<T>
	//       │
	//       └──> shared access to T
	//
	// We can read the value from multiple threads:
	//
	//     println!("{}", c1);
	//
	// But we cannot mutate the value:
	//
	//     c1 += 1; // ❌ cannot mutate through Arc
	//
	// Arc solves "how can multiple threads own the same data?"
	// It does NOT solve "how can multiple threads mutate that data?"
	let counter = Arc::new(0);
	let c1 = Arc::clone(&counter);
	let c2 = Arc::clone(&counter);
	let t1 = thread::spawn(move || {
		println!("thread 1: {}", c1);
		// c1 += 1; // ❌
	});
	let t2 = thread::spawn(move || {
		println!("thread 2: {}", c2);
		// c2 += 100; // ❌
	});
	t1.join().unwrap();
	t2.join().unwrap();
}
fn mutex_race_depends_on_scheduling() {
	// Arc<T>
	//     ↓
	// "I want multiple threads to own/read the same T"
	// let counter = Arc::new(0);
	// Arc<Mutex<T>>
	//     ↓
	// "I want multiple threads to safely mutate the same T"
	let counter = Arc::new(Mutex::new(0));
	//             Arc
	//              │
	//              ▼
	//        ┌─────────────┐
	//        │ Mutex<i32>  │
	//        │      0      │
	//        └─────────────┘
	//          ▲         ▲
	//          │         │
	//         c1         c2
	//       thread 1   thread 2
	let c1 = Arc::clone(&counter);
	let c2 = Arc::clone(&counter);
	let t1 = thread::spawn(move || {
		// The Mutex protects each individual read/write,
		// but NOT the entire read → modify → write operation.
		let current = *c1.lock().unwrap(); // LOCK → READ → UNLOCK
		// Give the other thread a chance to run after we've
		// read the value but before we've written it.
		thread::sleep(Duration::from_millis(10));
		*c1.lock().unwrap() = current + 1; // LOCK → WRITE → UNLOCK
	});
	let t2 = thread::spawn(move || {
		let current = *c2.lock().unwrap(); // LOCK → READ → UNLOCK
		thread::sleep(Duration::from_millis(10));
		*c2.lock().unwrap() = current + 100; // LOCK → WRITE → UNLOCK
	});
	t1.join().unwrap();
	t2.join().unwrap();
	println!("mutex_race_depends_on_scheduling");
	println!("counter = {}", *counter.lock().unwrap());
}

fn mutex_race_is_deterministically_reproduced() {
	let counter = Arc::new(Mutex::new(0));
	// Force both threads to finish their READ before either
	// thread is allowed to perform its WRITE.
	//
	// This makes the race deterministic instead of relying
	// on the OS scheduler to happen to interleave the threads.
	let barrier = Arc::new(Barrier::new(2));
	let c1 = Arc::clone(&counter);
	let c2 = Arc::clone(&counter);
	let b1 = Arc::clone(&barrier);
	let b2 = Arc::clone(&barrier);

	let t1 = thread::spawn(move || {
		let current = *c1.lock().unwrap(); // LOCK → READ → UNLOCK
		// Wait until thread 2 has also read the counter.
		b1.wait();
		// Both threads now have the same stale value.
		*c1.lock().unwrap() = current + 1; // LOCK → WRITE → UNLOCK
	});
	let t2 = thread::spawn(move || {
		let current = *c2.lock().unwrap(); // LOCK → READ → UNLOCK
		// Wait until thread 1 has also read the counter.
		b2.wait(); // "Don't let either thread continue until both threads have finished this phase."
		// Both threads now have the same stale value.
		*c2.lock().unwrap() = current + 100; // LOCK → WRITE → UNLOCK
	});
	t1.join().unwrap();
	t2.join().unwrap();
	println!("mutex_race_is_deterministically_reproduced");
	println!("counter = {}", *counter.lock().unwrap());
}
