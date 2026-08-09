use resvg;
use std::{env, fs, path::PathBuf};

fn main() {
	println!("cargo:rerun-if-changed=assets/estate.svg");
	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
	let output = out_dir.join("estate-tray.png");
	let svg = fs::read("assets/estate.svg").unwrap();
	let options = resvg::usvg::Options::default();
	let tree = resvg::usvg::Tree::from_data(&svg, &options).expect("failed to parse estate.svg");
	let size = tree.size().to_int_size();
	let mut pixmap =
		resvg::tiny_skia::Pixmap::new(size.width(), size.height()).expect("failed to create pixmap");
	resvg::render(
		&tree,
		resvg::tiny_skia::Transform::default(),
		&mut pixmap.as_mut(),
	);
	pixmap.save_png(output).unwrap();
}
