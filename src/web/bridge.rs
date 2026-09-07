use crate::prelude::*;

use crate::ui::*;
use eframe::{self, Frame, WebOptions, WebRunner};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use web_sys::js_sys;

/// Declares external functions using the C ABI.
/// wasm-bindgen uses these declarations to generate the Rust ↔ JavaScript bridge.
///
/// WASM build runs in the browser so do not expect to see "server logs" when you run the build using
/// trunk serve.
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
pub enum Status {
	Created,
	Running,
	Completed,
	Failed { message: String },
}

#[wasm_bindgen]
pub fn evaluate_ui(canvas: web_sys::HtmlCanvasElement) {
	let _rect = canvas.get_bounding_client_rect();
}

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
			},
		],
	};
	serde_wasm_bindgen::to_value(&payload).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen]
pub fn receive_payload(value: JsValue) -> Result<(), JsValue> {
	let payload: Payload =
		serde_wasm_bindgen::from_value(value).map_err(|err| JsValue::from_str(&err.to_string()))?;

	web_sys::console::log_1(&JsValue::from_str(&format!(
		"[RUST] RECEIVED:\n{payload:#?}"
	)));

	// Serialize Rust back into JavaScript.
	let response =
		serde_wasm_bindgen::to_value(&payload).map_err(|err| JsValue::from_str(&err.to_string()))?;

	js_test(response);

	Ok(())
}

#[wasm_bindgen]
pub fn install_api() {
	let window = web_sys::window().expect("no window");

	let receive = wasm_bindgen::closure::Closure::wrap(Box::new(move |value: JsValue| {
		if let Err(error) = receive_payload(value) {
			web_sys::console::error_1(&error);
		}
	}) as Box<dyn FnMut(JsValue)>);

	js_sys::Reflect::set(
		&window,
		&JsValue::from_str("receive_payload"),
		receive.as_ref().unchecked_ref(),
	)
	.expect("failed to install receive_payload");

	receive.forget();
}

pub fn spawn_clock(msg: String) {
	wasm_bindgen_futures::spawn_local(async move {
		loop {
			log(&format!("🔥 Tran Tran Clock run_background {msg}"));
			gloo_timers::future::TimeoutFuture::new(1000).await;
		}
	});
}

/// ## Wasm Entrypoint
///
/// Don't try to call this, the `wasm_bindgen(start)` macro does it automatically.
///
/// Just make sure you include this mod in the bin which is built for web.
#[wasm_bindgen(start)]
pub fn start() {
	install_api();

	wasm_bindgen_futures::spawn_local(async {
		log("🔥 RUST START() RUNNING");

		let payload = create_payload().expect("failed to create payload");
		js_test(payload);

		let document = web_sys::window()
			.expect("no window")
			.document()
			.expect("no document");

		let canvas = document
			.get_element_by_id("the_canvas_id")
			.expect("canvas not found")
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.expect("not a canvas");

		WebRunner::new()
			.start(
				canvas,
				WebOptions::default(),
				Box::new(|_cc| {
					// log("🔥 EFRAME APP CREATOR RUNNING");
					let host = Host::init()?;
					let mut app = App::new(host)?;
					// let screen = MarkdownScreen::new(include_str!("../data/corpus.md").to_owned());
					// 	Ok(Box::new(WebApp { graphics: screen }))
					app.start()?;
					// log("🔥 APP Loi");
					Ok(Box::new(app))
				}),
			)
			.await
			.expect("failed to start eframe");
	});
}

impl<C> eframe::App for App<C>
where
	C: AppCtx,
{
	fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {}
	fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {}
}
impl eframe::App for WebApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ui, |ui| {
			self.graphics.draw(ui);
		});
	}
}
impl HostClock {
	pub fn inherent_background_tick(&self, msg: String) {
		wasm_bindgen_futures::spawn_local(async move {
			loop {
				log(&format!("🔥 INHERENT TICK {msg}"));
				gloo_timers::future::TimeoutFuture::new(1000).await;
			}
		});
	}
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

struct WebApp {
	graphics: MarkdownScreen,
}
