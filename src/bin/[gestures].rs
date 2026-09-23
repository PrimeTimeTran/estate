// #![cfg(target_os = "macos")]

// use std::ffi::c_void;
// use std::ptr;

// type MTDeviceRef = *mut c_void;

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// struct MTPoint {
// 	x: f32,
// 	y: f32,
// }

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// struct MTVector {
// 	x: f32,
// 	y: f32,
// }

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// struct Finger {
// 	frame: u32,
// 	timestamp: f32,
// 	identifier: i32,

// 	state: i32,

// 	unknown1: i32,
// 	unknown2: i32,

// 	normalized: MTPoint,
// 	size: f32,

// 	zero1: f32,
// 	zero2: f32,

// 	angle: f32,
// 	major_axis: f32,
// 	minor_axis: f32,

// 	unknown3: MTVector,

// 	pressure: f32,

// 	velocity: MTVector,

// 	density: f32,

// 	unknown4: i32,
// 	unknown5: i32,
// 	unknown6: i32,
// }

// type TouchCallback = extern "C" fn(
// 	device: MTDeviceRef,
// 	timestamp: f64,
// 	touches: *const Finger,
// 	num_touches: usize,
// 	user_data: *mut c_void,
// );

// #[link(name = "MultitouchSupport", kind = "framework")]
// unsafe extern "C" {
// 	fn MTDeviceCreateDefault() -> MTDeviceRef;

// 	fn MTDeviceStart(device: MTDeviceRef, mode: i32) -> i32;

// 	fn MTDeviceSetContactFrameCallback(
// 		device: MTDeviceRef,
// 		callback: TouchCallback,
// 		user_data: *mut c_void,
// 	);
// }

// use std::sync::Mutex;

// #[derive(Debug, Default)]
// struct GestureState {
// 	one_finger_active: bool,
// 	two_finger_active: bool,

// 	last_y: Option<f32>,
// }

// static STATE: Mutex<GestureState> = Mutex::new(GestureState {
// 	one_finger_active: false,
// 	two_finger_active: false,
// 	last_y: None,
// });

// extern "C" fn touch_callback(
// 	_device: MTDeviceRef,
// 	_timestamp: f64,
// 	touches: *const Finger,
// 	num_touches: usize,
// 	_user_data: *mut c_void,
// ) {
// 	let touches = unsafe { std::slice::from_raw_parts(touches, num_touches) };

// 	let mut state = STATE.lock().unwrap();

// 	match touches.len() {
// 		0 => {
// 			state.one_finger_active = false;
// 			state.two_finger_active = false;
// 			state.last_y = None;
// 		}

// 		1 => {
// 			// First finger is down.
// 			state.one_finger_active = true;
// 			state.two_finger_active = false;
// 			state.last_y = None;
// 		}

// 		2.. => {
// 			// At least two fingers are currently touching.
// 			if !state.one_finger_active {
// 				return;
// 			}

// 			state.two_finger_active = true;

// 			// Average Y position of the additional fingers.
// 			let y = touches
// 				.iter()
// 				.skip(1)
// 				.map(|finger| finger.normalized.y)
// 				.sum::<f32>()
// 				/ (touches.len() - 1) as f32;

// 			if let Some(previous_y) = state.last_y {
// 				let delta = y - previous_y;

// 				if delta.abs() > 0.002 {
// 					if delta > 0.0 {
// 						println!("ESTATE: two-finger scroll DOWN");
// 					} else {
// 						println!("ESTATE: two-finger scroll UP");
// 					}
// 				}
// 			}

// 			state.last_y = Some(y);
// 		}

// 		_ => {}
// 	}
// }

// pub fn main() {
// 	unsafe {
// 		let device = MTDeviceCreateDefault();

// 		if device.is_null() {
// 			panic!("Could not create trackpad device");
// 		}

// 		MTRegisterContactFrameCallback(device, touch_callback, ptr::null_mut());

// 		let result = MTDeviceStart(device, 0);

// 		if result != 0 {
// 			panic!("MTDeviceStart failed: {result}");
// 		}
// 	}

// 	loop {
// 		std::thread::park();
// 	}
// }
#![cfg(target_os = "macos")]

use std::{
	ffi::c_int,
	time::{Duration, Instant},
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct EstateTouch {
	id: c_int,
	state: c_int,

	x: f32,
	y: f32,

	size: f32,
	major_axis: f32,
	minor_axis: f32,
	angle: f32,

	velocity_x: f32,
	velocity_y: f32,
}

unsafe extern "C" {
	fn estate_multitouch_start(
		callback: extern "C" fn(*const EstateTouch, c_int, f64, c_int),
	) -> c_int;
}

static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

extern "C" fn callback(touches: *const EstateTouch, count: c_int, timestamp: f64, frame: c_int) {
	let start = START.get_or_init(Instant::now);

	let elapsed = start.elapsed();

	let touches = unsafe { std::slice::from_raw_parts(touches, count as usize) };

	println!(
		"\n[{:.3}s] frame={} fingers={}",
		elapsed.as_secs_f64(),
		frame,
		touches.len(),
	);

	for (i, touch) in touches.iter().enumerate() {
		println!(
			"  {:>2}: id={:<3} state={:<2} \
             x={:.3} y={:.3} \
             vx={:+.3} vy={:+.3} \
             size={:.3}",
			i, touch.id, touch.state, touch.x, touch.y, touch.velocity_x, touch.velocity_y, touch.size,
		);
	}
}

pub fn main() {
	START.set(Instant::now()).ok();

	let result = unsafe { estate_multitouch_start(callback) };

	if result != 0 {
		panic!("Multitouch initialization failed: {result}");
	}

	println!("Trackpad monitor running.");
	println!("Put fingers on the trackpad...\n");

	loop {
		std::thread::sleep(Duration::from_secs(1));
	}
}
