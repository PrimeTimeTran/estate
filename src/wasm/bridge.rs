use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::from_value;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use web_sys::js_sys;

#[wasm_bindgen]
pub fn test_wasm() -> String {
	"Hello from Rust".to_owned()
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

	pub metadata: HashMap<String, String>,

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
	Failed { message: String },
}

#[wasm_bindgen]
pub fn create_payload() -> Result<JsValue, JsValue> {
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
		metadata: HashMap::from([
			("environment".into(), "development".into()),
			("platform".into(), "wasm".into()),
		]),
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
			},
		],
	};

	serde_wasm_bindgen::to_value(&payload).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn receive_payload(value: JsValue) -> Result<(), JsValue> {
	let payload: Payload = from_value(value).map_err(|err| JsValue::from_str(&err.to_string()))?;

	web_sys::console::log_1(&JsValue::from_str(&format!("RUST RECEIVED:\n{payload:#?}")));

	Ok(())
}

// #[wasm_bindgen(start)]
// pub fn start() -> Result<(), JsValue> {
// 	web_sys::console::log_1(&JsValue::from_str("RUST STARTED"));

// 	Ok(())
// }

// #[wasm_bindgen]
// pub fn get_state() -> Result<JsValue, JsValue> {
//     let state = estate_core::state();

//     serde_wasm_bindgen::to_value(&state)
//         .map_err(|e| JsValue::from_str(&e.to_string()))
// }

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);

	fn js_test(payload: JsValue);
}

// #[wasm_bindgen]
// pub fn run_test(callback: js_sys::Function) -> Result<(), JsValue> {
// 	log("RUST RUNNING");

// 	let payload = create_payload()?;

// 	callback.call1(&JsValue::NULL, &payload)?;

// 	Ok(())
// }

// #[wasm_bindgen(start)]
// pub fn start() -> Result<(), JsValue> {
// 	log("RUST STARTED");

// 	let payload = create_payload()?;

// 	js_test(payload);

// 	Ok(())
// }
