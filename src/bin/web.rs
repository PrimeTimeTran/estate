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
// "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
#[cfg(target_arch = "wasm32")]
mod wasm {
	use eframe::{ WebOptions, WebRunner };
	use estate::{ share::ve::*, web::bridge::* };
	use wasm_bindgen::{ prelude::*, JsCast };

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
			js_test(payload);

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
						log("🔥 EFRAME APP CREATOR RUNNING");

						Ok(
							Box::new(WebApp {
								graphics: Graphics::new(),
							})
						)
					})
				).await
				.expect("failed to start eframe");

			log("🔥 EFRAME START RETURNED");
		});
	}

	#[wasm_bindgen]
	pub fn evaluate_ui(canvas: web_sys::HtmlCanvasElement) {
		let _rect = canvas.get_bounding_client_rect();
	}
}

#[cfg(target_arch = "wasm32")]
fn main() {
	todo!("main")
}

// #[cfg(feature = "web")]
#[cfg(not(target_arch = "wasm32"))]
fn main() {}
