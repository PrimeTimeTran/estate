// $ cargo build --bin web --no-default-features --features="web" --target wasm32-unknown-unknown
// $ trunk serve src/web/public/index.html --features web
//
// cargo build \
//   --target wasm32-unknown-unknown \
//   --bin web \
//   --no-default-features \
//   --features web
//
//   cargo build --bin web --no-default-features --features="web" --target wasm32-unknown-unknown && trunk serve src/web/public/index.html --features web
//
// Debug Build
// cargo tree \
//   --bin web \
//   --target wasm32-unknown-unknown \
//   -e features
//
// "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
// #[cfg(target_arch = "wasm32")]
// mod wasm {
// use eframe::{WebOptions, WebRunner};
// use estate::{
// ui::{layout::*, *},
// web::bridge::*,
// };
// use wasm_bindgen::{JsCast, prelude::*};
//
// struct WebApp {
// graphics: MarkdownView,
// }
//
// impl eframe::App for WebApp {
// fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
// // egui::CentralPanel::default().show(ui, |ui| {
// // 	self.graphics.draw(ui);
// // });
// }
// }
//
// #[wasm_bindgen(start)]
// pub fn start() {
// install_api();
//
// wasm_bindgen_futures::spawn_local(async {
// log("🔥 RUST START() RUNNING");
//
// let payload = create_payload().expect("failed to create payload");
// js_test(payload);
//
// let document = web_sys::window()
// .expect("no window")
// .document()
// .expect("no document");
//
// let canvas = document
// .get_element_by_id("the_canvas_id")
// .expect("canvas not found")
// .dyn_into::<web_sys::HtmlCanvasElement>()
// .expect("not a canvas");
//
// WebRunner::new()
// .start(
// canvas,
// WebOptions::default(),
// Box::new(|_cc| {
// log("🔥 EFRAME APP CREATOR RUNNING");
//
// Ok(Box::new(WebApp {
// graphics: MarkdownView::new(
// "/Users/future/kb/project/crates/estate/src/data/corpus.md",
// ), // graphics: Ve::new(MarkdownView::new(
// // 	"/Users/future/kb/project/crates/estate/src/data/corpus.md",
// // )),
// }))
// }),
// )
// .await
// .expect("failed to start eframe");
//
// log("🔥 EFRAME START RETURNED");
// });
// }
//
// #[wasm_bindgen]
// pub fn evaluate_ui(canvas: web_sys::HtmlCanvasElement) {
// let _rect = canvas.get_bounding_client_rect();
// }
// }

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
	let app = App::new().map_err(|e| JsValue::from_str(&e.to_string()))?;

	// create WasmRuntime
	// create EstateEngine<WasmRuntime>
	// create AppRuntime<WasmRuntime>
	// hand it to your UI

	Ok(())
}

use estate::{app::state::EstateState, app::*};
#[cfg(target_arch = "wasm32")]
fn main() {
	let runtime = WasmRuntime::new(EstateState::default());
	let app = AppRuntime::new(engine, api);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}

use std::{
	future::Future,
	sync::{Arc, RwLock},
	time::Duration,
};

use async_trait::async_trait;
use tokio::sync::broadcast;

// pub type NativeAppRuntime =
//     AppRuntime<NativeRuntime, NativeApiClient>;

// pub type WasmAppRuntime = AppRuntime<WasmRuntime, WasmApiClient>;
pub struct WasmRuntime {
	state: Arc<RwLock<EstateState>>,
	events: broadcast::Sender<e::Event>,
}

impl WasmRuntime {
	pub fn new(state: EstateState) -> Self {
		let (events, _) = broadcast::channel(256);

		Self {
			state: Arc::new(RwLock::new(state)),
			events,
		}
	}
}

impl Runtime for WasmRuntime {
	type EventReceiver = broadcast::Receiver<e::Event>;

	fn emit(&self, event: e::Event) {
		// Ignore error when there are no subscribers.
		let _ = self.events.send(event);
	}

	fn subscribe(&self) -> Self::EventReceiver {
		self.events.subscribe()
	}

	fn state(&self) -> Arc<RwLock<EstateState>> {
		Arc::clone(&self.state)
	}

	fn spawn<F>(&self, future: F)
	where
		F: Future<Output = ()> + 'static,
	{
		wasm_bindgen_futures::spawn_local(future);
	}

	async fn sleep(&self, duration: Duration) {
		gloo_timers::future::sleep(duration).await;
	}
}
