/// ### Rust Ownership & Conversion Cheat sheet

// ==========================================
// RUST OWNERSHIP & CONVERSION METHODS CHEAT SHEET
// ==========================================

// --- CATEGORY 1: CONSUMING & MOVING ---

// 1. .into()
// Converts a value into another type by consuming ownership. Often used for
// ergonomic function arguments (e.g., impl Into<PathBuf>).

fn take_path(path: impl Into<std::path::PathBuf>) {
	let _p: std::path::PathBuf = path.into();
}

// 2. .take() (std::option::Option / std::cell::Cell)
// Replaces the value inside an Option or Cell with `None`, returning the original
// value. Essential for moving out of struct fields without breaking ownership.
fn extract_rx(mut rx_opt: Option<i32>) -> i32 {
	rx_opt.take().expect("already taken")
}

// 3. .unwrap() / .expect()
// Consumes an Option or Result, taking ownership of the inner value and panicking
// on None/Err.
fn get_value(val: Option<String>) -> String {
	val.unwrap()
}

// 4. .drain()
// Removes a specified range from a collection, returning an iterator that yields
// those owned elements while removing them from the original collection.
fn drain_vec(mut v: Vec<i32>) -> Vec<i32> {
	v.drain(1..3).collect()
}

// 5. mem::take()
// Replaces a mutable value with its type's default value, returning the old owned value.
fn take_string_field(s: &mut String) -> String {
	std::mem::take(s)
}

// 6. mem::replace()
// Replaces a mutable value with a new value you provide, returning the old owned value.
fn swap_string(s: &mut String) {
	let _old = std::mem::replace(s, String::from("new default"));
}

// --- CATEGORY 2: CLONING & DUPLICATING ---

// 7. .clone()
// Deep-copies a value, creating a brand new owned instance on the heap/stack.
fn duplicate(s: String) -> (String, String) {
	let s2 = s.clone();
	(s, s2)
}

// 8. .to_owned()
// Converts a borrowed slice or str into an owned collection (e.g., &str -> String, &[T] -> Vec<T>).
fn make_owned(slice: &str) -> String {
	slice.to_owned()
}

// 9. .to_string()
// Specifically converts anything implementing `Display` into an owned `String`.
fn stringify(n: i32) -> String {
	n.to_string()
}

// 10. .to_vec()
// Converts a slice into an owned `Vec<T>` by cloning elements.
fn clone_slice(slice: &[i32]) -> Vec<i32> {
	slice.to_vec()
}

// --- CATEGORY 3: BORROWING & REFERENCES ---

// 11. .as_ref()
// Converts a wrapper type (like Option<T>, Result<T, E>, or Box<T>) containing an
// owned value into a wrapper containing a reference (Option<&T>).
fn check_opt(opt: Option<String>) -> Option<usize> {
	opt.as_ref().map(|s| s.len()) // 'opt' is consumed, but 's' is borrowed
}

// 12. .as_mut()
// Same as as_ref(), but yields a mutable reference (&mut T) instead of a shared one.
fn mutate_opt(opt: &mut Option<String>) {
	if let Some(s) = opt.as_mut() {
		s.push_str(" appended");
	}
}

// 13. .borrow() (std::borrow::Borrow trait)
// Used in collections (like HashMap/BTreeMap) to allow querying with a borrowed
// type (e.g., querying a `HashMap<String, V>` using a `&str`).
// (Called implicitly by APIs like map.get("key"))

// 14. .deref() (std::ops::Deref trait)
// Automatically coerces a smart pointer (like `Box<T>`, `Arc<T>`, `String`) into
// a reference to its inner data (`&T`, `&str`). Usually done implicitly via Deref Coercion.
fn inspect_str(s: &String) {
	let _slice: &str = s; // Deref coercion from &String to &str
}

// --- CATEGORY 4: CONVERTING POINTERS & WRAPPERS ---

// 15. Arc::try_unwrap()
// Attempts to extract the inner value from an `Arc` if and only if there is
// exactly 1 strong reference left. Consumes the Arc.
fn get_unique_from_arc(arc: std::sync::Arc<String>) -> Result<String, std::sync::Arc<String>> {
	std::sync::Arc::try_unwrap(arc)
}

// 16. Box::into_inner()
// Extracts the inner value from a `Box<T>` without deallocating if possible (Rust 1.72+).
fn unbox(b: Box<String>) -> String {
	Box::into_inner(b)
}

// 17. Box::leak()
// Consumes a Box and forces it to live for the `'static` lifetime by leaking memory,
// returning a mutable reference (`&'static mut T`).
fn leak_data(s: String) -> &'static mut str {
	Box::leak(s.into_boxed_str())
}

// 18. .iter() / .into_iter() / .iter_mut()
// - iter(): Yields immutable borrows (&T)
// - into_iter(): Consumes the collection, yielding owned items (T)
// - iter_mut(): Yields mutable borrows (&mut T)
fn process_collection(v: Vec<i32>) {
	for _item in v {
		// into_iter() is called implicitly here, consuming 'v'
	}
}

// --- CATEGORY 5: SMART POINTER SAFETY & CONVERSIONS ---

// 19. Rc::clone() / Arc::clone()
// Cheaply increments the reference count of a shared smart pointer rather than
// deep-cloning the inner data.
fn share_arc(data: std::sync::Arc<i32>) -> (std::sync::Arc<i32>, std::sync::Arc<i32>) {
	let clone = std::sync::Arc::clone(&data);
	(data, clone)
}

// 20. .as_deref()
// Converts an Option<String> into an Option<&str> (or Option<Vec<T>> to Option<&[T]>).
fn view_optional_string(opt: &Option<String>) -> Option<&str> {
	opt.as_deref()
}

// 21. .map()
// Transforms the contents of an Option, Result, or Iterator by closure,
// consuming the wrapper's inner value and returning a new wrapped value.
fn transform(opt: Option<String>) -> Option<usize> {
	opt.map(|s| s.len())
}

// 22. .and_then()
// Similar to map, but the closure returns another wrapped type, flattening nested wrappers
// (like Option<Option<T>> -> Option<T>).
fn chain_options(opt: Option<String>) -> Option<char> {
	opt.and_then(|s| s.chars().next())
}

// 23. .ok()
// Converts a `Result<T, E>` into an `Option<T>`, discarding the error variant.
fn result_to_option(res: Result<i32, String>) -> Option<i32> {
	res.ok()
}

// 24. .collect::<CollectionType>()
// Transforms an iterator into an owned collection (like Vec, HashSet, HashMap)
// by consuming the iterator items.
fn vec_to_map(v: Vec<(i32, String)>) -> std::collections::HashMap<i32, String> {
	v.into_iter().collect()
}

// 25. std::convert::TryInto::try_into()
// Fallible version of `.into()`. Returns a `Result`, useful for type casting
// (e.g., converting u64 to u32 where overflow could fail).
fn safe_cast(val: u64) -> Result<u32, std::num::TryFromIntError> {
	val.try_into()
}

// ```

// ---

// ### Follow-Up Question

// When working with your window and widget loops, do you find yourself needing to share state across multiple threads more often via `Arc<Mutex<T>>`, or do you prefer passing messages through channels?
