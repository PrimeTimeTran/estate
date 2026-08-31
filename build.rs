use resvg;
use std::{env, fs, path::PathBuf};

// Generate:
// $ cargo build
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
	let output = out_dir.join("estate-tray.png");
	let svg = fs::read_to_string("assets/estate.svg").unwrap();
	let color = env::var("ESTATE_ICON_COLOR").unwrap_or_else(|_| "#000000".to_string());
	let svg = svg.replace("#ESTATE_COLOR", &color);
	let svg = svg.replace("#FFFFFF", "#374957");
	let options = resvg::usvg::Options::default();
	let tree = resvg::usvg::Tree::from_str(&svg, &options).expect("failed to parse estate.svg");
	let size = tree.size().to_int_size();
	let mut pixmap =
		resvg::tiny_skia::Pixmap::new(size.width(), size.height()).expect("failed to create pixmap");
	resvg::render(
		&tree,
		resvg::tiny_skia::Transform::default(),
		&mut pixmap.as_mut(),
	);
	pixmap.save_png(output).unwrap();
	println!("cargo:rerun-if-changed=proto/service.proto");
	println!("cargo:rerun-if-changed=proto/type.proto");
	tonic_build::compile_protos("proto/service.proto")?;
	Ok(())
}
