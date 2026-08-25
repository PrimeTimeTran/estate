use eframe::{WebOptions, WebRunner, egui::*};
use estate::share::ve::*;
use wasm_bindgen::{JsCast, prelude::*};

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
	wasm_bindgen_futures::spawn_local(async {
		let document = web_sys::window()
			.expect("no window")
			.document()
			.expect("no document");

		let canvas = document
			.get_element_by_id("the_canvas_id")
			.expect("canvas #the_canvas_id not found")
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.expect("#the_canvas_id is not a canvas");

		WebRunner::new()
			.start(
				canvas,
				WebOptions::default(),
				Box::new(|_cc| {
					Ok(Box::new(WebApp {
						graphics: Graphics::new(),
					}))
				}),
			)
			.await
			.expect("failed to start eframe");
	});
}

fn main() {}
