use crate::app::*;

use serde::{ Deserialize, Serialize };
use wasm_bindgen::{ prelude::*, JsValue };
use web_sys::js_sys;

#[wasm_bindgen]
pub fn create_payload() -> Result<JsValue, JsValue> {
	let meta_hashmap = HashMap::from([
		("environment".into(), "development".into()),
		("platform".into(), "wasm".into()),
	]);
	let meta_hashset: HashSet<(String, String)> = HashSet::from([
		("environment".into(), "development".into()),
		("platform".into(), "wasm".into()),
		("version".into(), "0.1.0".into()),
	]);
	let payload = Payload {
		id: 123456789,
		name: "Estate".into(),
		active: true,
		count: -42,
		score: 98.5,
		price: 1234.5678,
		optional: Some("hello from Rust".into()),
		tags: vec!["rust".into(), "wasm".into(), "egui".into()],
		values: vec![-100, 0, 42, 999999],
		meta_hashmap,
		meta_hashset,
		bytes: vec![0, 1, 2, 127, 255],
		address: Address {
			street: "123 Main St".into(),
			city: "Jacksonville".into(),
			zip: 32202,
		},
		status: Status::Running,
		children: vec![
			Child {
				id: 1,
				name: "First".into(),
				enabled: true,
			},
			Child {
				id: 2,
				name: "Second".into(),
				enabled: false,
			}
		],
	};
	serde_wasm_bindgen::to_value(&payload).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn receive_payload(value: JsValue) -> Result<(), JsValue> {
	let payload: Payload = serde_wasm_bindgen
		::from_value(value)
		.map_err(|err| JsValue::from_str(&err.to_string()))?;

	web_sys::console::log_1(&JsValue::from_str(&format!("[RUST] RECEIVED:\n{payload:#?}")));

	// Serialize Rust back into JavaScript.
	let response = serde_wasm_bindgen
		::to_value(&payload)
		.map_err(|err| JsValue::from_str(&err.to_string()))?;

	js_test(response);

	Ok(())
}

#[wasm_bindgen]
pub fn install_api() {
	let window = web_sys::window().expect("no window");

	let receive = wasm_bindgen::closure::Closure::wrap(
		Box::new(move |value: JsValue| {
			if let Err(error) = receive_payload(value) {
				web_sys::console::error_1(&error);
			}
		}) as Box<dyn FnMut(JsValue)>
	);

	js_sys::Reflect
		::set(&window, &JsValue::from_str("receive_payload"), receive.as_ref().unchecked_ref())
		.expect("failed to install receive_payload");

	receive.forget();
}

// Declares external functions using the C ABI.
// wasm-bindgen uses these declarations to generate the Rust ↔ JavaScript bridge.
#[wasm_bindgen]
extern "C" {
	// This declares a Rust function that calls the JavaScript
	// function `js_test(payload)`. It does NOT define `window.js_test`.
	#[wasm_bindgen(js_name = js_test)]
	pub fn js_test(payload: JsValue);
	// This declares a Rust function that calls `console.log(s)`.
	#[wasm_bindgen(js_namespace = console)]
	pub fn log(s: &str);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
	pub id: u64,
	pub name: String,
	pub active: bool,

	pub count: i32,
	pub score: f32,
	pub price: f64,

	pub optional: Option<String>,

	pub tags: Vec<String>,
	pub values: Vec<i64>,

	pub meta_hashmap: HashMap<String, String>,
	pub meta_hashset: HashSet<(String, String)>,

	pub bytes: Vec<u8>,

	pub address: Address,

	pub status: Status,

	pub children: Vec<Child>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
	pub street: String,
	pub city: String,
	pub zip: u32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Child {
	pub id: u32,
	pub name: String,
	pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
	Created,
	Running,
	Completed,
	Failed {
		message: String,
	},
}
