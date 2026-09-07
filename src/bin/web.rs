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

use estate::*;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
mod wasm {
	use eframe::{WebOptions, WebRunner};
	use estate::{ui::*, web::bridge::*};
	use wasm_bindgen::{JsCast, prelude::*};

	struct WebApp {
		graphics: MarkdownScreen,
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
						log("🔥 EFRAME APP CREATOR RUNNING");
						let screen = MarkdownScreen::new(include_str!("../data/corpus.md").to_owned());
						Ok(Box::new(WebApp { graphics: screen }))
					}),
				)
				.await
				.expect("failed to start eframe");

			log("🔥 EFRAME START RETURNED");
		});
	}

	#[wasm_bindgen]
	pub fn evaluate_ui(canvas: web_sys::HtmlCanvasElement) {
		let _rect = canvas.get_bounding_client_rect();
	}
}

#[cfg(not(feature = "web"))]
#[tokio::main]
fn main() -> anyhow::Result<()> {
	let host = Host::init()?;
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}

fn main() -> anyhow::Result<()> {
	let host = Host::init()?;
	let mut app = App::new(host)?;
	app.run()?;
	Ok(())
}
