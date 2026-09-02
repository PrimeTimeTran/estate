// https://www.youtube.com/watch?v=SbcYv5EQGNM&t=32s
// https://www.youtube.com/watch?v=rNhfQimiwEs
// https://www.youtube.com/watch?v=8gNKE5jVqYY
#![allow(warnings)]
use std::{
	sync::{
		Arc, Barrier, Mutex, RwLock,
		atomic::{AtomicUsize, Ordering},
	},
	thread,
	time::Duration,
};
fn main() {
	threads_are_async();
	demo_loop();
	threads_have_non_deterministic_lifetimes();
	move_transfers_ownership();
	arc_handles_ownership();
	arc_shares_ownership_across_threads();
	arc_mutex_allows_shared_mutation();
	mutex_does_not_make_separate_operations_atomic();
	rw_lock_serializes_mutable_access();
	println!("{:?} Main Thread done", thread::current().id());
}
fn threads_are_async() {
	let t2 = thread::spawn(|| {
		/// Both main and this function/thread are executed asynchronously
		/// so the following prints are not guaranteed to print
		///
		/// A call to `thread::spawn()` returns a handle which can be used to ensure
		/// that this thread must complete before it's parent process can exit.
		/// t2.join().unwrap();
		println!("{:?} Spawned thread '2' running...", thread::current().id());
		println!("{:?} thread '2' done", thread::current().id())
	});
	// t2.join().unwrap();
	println!("{:?} Spawned thread '2' done", thread::current().id());
}
fn demo_loop() {
	let t3 = thread::spawn(|| {
		for i in 1..3 {
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
fn threads_have_non_deterministic_lifetimes() {
	/// [Error]: "Closure may outlive the current function"
	/// Solve this by moving all the closure's dependencies ownership using `move`
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
	// [Error]: use of moved value: `s`
	// [when] let s = String::from("hello");
	// - "s" owns owns a heap-allocated String.
	//
	// [Solution]: &str
	// &'static str is it's type
	// &str is a ref to string data and string literals are stored in the program's binary and live for the entire program.
	// let s = String::from("hello");
	let s = "hello";
	let t5 = thread::spawn(move || {
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
fn arc_shares_ownership_across_threads() {
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
		// cannot assign to data in an `Arc`
		// trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Arc<i32>`
		// *c1 += 1;
		// binary assignment operation += cannot be applied to type Arc<{integer}> (rustc E0368)
		// c1 += 1;
	});
	let t2 = thread::spawn(move || {
		println!("thread 2: {}", c2);
		// cannot assign to data in an `Arc`
		// trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Arc<i32>`
		// *c1 += 100;
		// binary assignment operation += cannot be applied to type Arc<{integer}> (rustc E0368)
		// c1 += 100;
	});
	t1.join().unwrap();
	t2.join().unwrap();
}
fn arc_mutex_allows_shared_mutation() {
	let counter = Arc::new(Mutex::new(0));

	let c1 = Arc::clone(&counter);
	let c2 = Arc::clone(&counter);

	let t1 = thread::spawn(move || {
		*c1.lock().unwrap() += 1;
	});

	let t2 = thread::spawn(move || {
		*c2.lock().unwrap() += 100;
	});

	t1.join().unwrap();
	t2.join().unwrap();

	println!("arc_mutex_allows_shared_mutation");
	println!("counter = {}", *counter.lock().unwrap());
}
fn mutex_does_not_make_separate_operations_atomic() {
	// The Mutex protects each individual access,
	// but the read → modify → write sequence is split
	// across two separate lock acquisitions.
	let counter = Arc::new(Mutex::new(0));
	// Force both threads to complete their reads before
	// either thread is allowed to perform its write.
	//
	// This guarantees both threads operate on the same
	// stale value, making the lost update deterministic.
	let barrier = Arc::new(Barrier::new(3)); // Create a barrier that requires 3 threads to arrive before either one is allowed to continue.
	let c1 = Arc::clone(&counter);
	let c2 = Arc::clone(&counter);
	let c3 = Arc::clone(&counter);
	let b1 = Arc::clone(&barrier);
	let b2 = Arc::clone(&barrier);
	let b3 = Arc::clone(&barrier);

	let t1 = thread::spawn(move || {
		let current = *c1.lock().unwrap(); // LOCK → READ → UNLOCK
		b1.wait(); // "Don't let either thread continue until both threads have finished this phase."
		// Both threads now have the same stale value.
		*c1.lock().unwrap() = current + 1;
	});
	let t2 = thread::spawn(move || {
		let current = *c2.lock().unwrap(); // LOCK → READ → UNLOCK
		b2.wait(); // "Don't let either thread continue until both threads have finished this phase."
		// Both threads now have the same stale value.
		*c2.lock().unwrap() = current + 100;
	});
	let t3 = thread::spawn(move || {
		let current = *c3.lock().unwrap(); // LOCK → READ → UNLOCK
		b3.wait(); // "Don't let either thread continue until both threads have finished this phase."
		// Both threads now have the same stale value.
		*c3.lock().unwrap() = current + 1000;
	});
	t1.join().unwrap();
	t2.join().unwrap();
	t3.join().unwrap();
	println!("mutex_does_not_make_separate_operations_atomic");
	println!("counter = {}", *counter.lock().unwrap());
	// Arc<Mutex<i32>>
	//      │
	//      └── coordinates ACCESS TO DATA
	// Arc<Barrier>
	//      │
	//      └── coordinates PROGRESS OF THREADS
}
fn rw_lock_serializes_mutable_access() {
	// RwLock makes the entire mutable access through a write guard exclusive, so concurrent writers cannot operate on stale copies of the protected value.
	let counter = Arc::new(RwLock::new(0));
	let c1 = Arc::clone(&counter);
	let c2 = Arc::clone(&counter);
	let c3 = Arc::clone(&counter);
	let t1 = thread::spawn(move || {
		let mut num: std::sync::RwLockWriteGuard<'_, i32> = c1.write().unwrap();
		thread::sleep(Duration::from_millis(10));
		*num += 1;
	});
	let t2 = thread::spawn(move || {
		let mut num = c2.write().unwrap();
		thread::sleep(Duration::from_millis(10));
		*num += 100;
	});
	let t3 = thread::spawn(move || {
		let mut num = c3.write().unwrap();
		thread::sleep(Duration::from_millis(100));
		*num += 1000;
	});
	t1.join().unwrap();
	t2.join().unwrap();
	t3.join().unwrap();
	println!("rw_lock_serializes_mutable_access");
	println!("counter = {}", *counter.read().unwrap());
}

fn size() {
	// https://doc.rust-lang.org/rust-by-example/std/box.html
	// rustup doc
	//  - The Rust Programming Language
	//  - Std Lib
	//  - Rust by Reference
	//  - Rust by Example
	//  - Rustnomicon
	// rustup doc --book
	// - The Rust Programming Language
	// rustup doc --std
	// - Rust STD library Docs
	// rustup doc --reference
	// - Rust by Reference
	// rustup doc --rust-by-example
	// - Rustnomicon
	// - Unsafe Rust Book
	// rustup doc --nomicon
	// - Rustnomicon
	// - Unsafe Rust Book

	#![allow(warnings)]
	use std::mem;

	#[derive(Debug, Clone, Copy)]
	struct Point {
		x: f64,
		y: f64,
	}

	// A Rectangle can be specified by where its top left and bottom right
	// corners are in space
	struct Rectangle {
		top_left: Point,
		bottom_right: Point,
	}

	fn origin() -> Point {
		Point { x: 0.0, y: 0.0 }
	}

	fn boxed_origin() -> Box<Point> {
		// Allocate this point on the heap, and return a pointer to it
		Box::new(Point { x: 0.0, y: 0.0 })
	}

	fn main() {
		// T:
		// ┌──────────────────────┐
		// │ actual data          │
		// │ actual data          │
		// │ actual data          │
		// └──────────────────────┘

		// Box<T>:
		// ┌──────────────┐
		// │ address ─────────────→ T
		// └──────────────┘
		let point = origin();
		let rectangle = Rectangle {
			top_left: origin(),
			bottom_right: Point { x: 3.0, y: -4.0 },
		};

		// Heap allocated rectangle
		let boxed_rectangle = Box::new(Rectangle {
			top_left: origin(),
			bottom_right: Point { x: 3.0, y: -4.0 },
		});

		// The output of functions can be boxed
		let boxed_point = Box::new(origin());

		// Double indirection
		let box_in_a_box = Box::new(boxed_origin());

		println!(
			"Point occupies {} bytes on the stack",
			mem::size_of_val(&point)
		);
		println!(
			"Rectangle occupies {} bytes on the stack",
			mem::size_of_val(&rectangle)
		);

		// box size == pointer size
		println!(
			"Boxed point occupies {} bytes on the stack", // 8
			mem::size_of_val(&boxed_point)
		);
		println!(
			"Boxed rectangle occupies {} bytes on the stack",
			mem::size_of_val(&boxed_rectangle)
		);
		println!(
			"Boxed box occupies {} bytes on the stack",
			mem::size_of_val(&box_in_a_box)
		);

		// Copy the data contained in `boxed_point` into `unboxed_point`
		let unboxed_point: Point = *boxed_point;
		println!(
			"Unboxed point occupies {} bytes on the stack", // 16
			mem::size_of_val(&unboxed_point)
		);
	}
}
