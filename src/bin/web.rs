// $ cargo build --bin web --target wasm32-unknown-unknown
// $ trunk serve src/web/public/index.html --features web
//
// cargo build \
//   --target wasm32-unknown-unknown \
//   --bin web \
//   --no-default-features \
//   --features web
//
//   cargo build --bin web --target wasm32-unknown-unknown && trunk serve src/web/public/index.html --features web
//
// "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
use eframe::{ WebOptions, WebRunner };
use estate::{ share::ve::*, web::bridge::* };
use wasm_bindgen::{ JsCast, prelude::* };

struct WebApp {
	graphics: Graphics,
}

impl eframe::App for WebApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ui, |ui| {
			self.graphics.draw(ui);
		});
	}
}

#[wasm_bindgen(start)]
pub fn start() {
	install_api();
	wasm_bindgen_futures::spawn_local(async {
		log("🔥 RUST START() RUNNING");

		let payload = create_payload().expect("failed to create payload");

		log("🔥 ABOUT TO CALL js_test()");

		js_test(payload);

		log("🔥 js_test() RETURNED");

		let document = web_sys::window().expect("no window").document().expect("no document");

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
					Ok(
						Box::new(WebApp {
							graphics: Graphics::new(),
						})
					)
				})
			).await
			.expect("failed to start eframe");
	});
}
pub fn main() {}
