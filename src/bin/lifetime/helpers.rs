use owo_colors::OwoColorize;
use std::path::{ Path, PathBuf };

#[macro_export]
macro_rules! section {
	($title:expr) => {
		$crate::helpers::print_section(
			$title,
			file!(),
			line!(),
		)
	};
}

pub(crate) use section;

pub(crate) fn print_section(title: &str, file: &str, line: u32) {
	let file = normalize_file(file);
	println!("{}", "-".repeat(80).dimmed());
	println!("{}", title.bold().cyan());
	println!("See {}", hyperlink(&format!("{file}#{line}"), &source_url(&file, line)).dimmed());
	println!("{}", "-".repeat(80).dimmed());
}

pub(crate) fn hyperlink(text: &str, url: &str) -> String {
	format!("\x1b]8;;{url}\x07{text}\x1b]8;;\x07")
}

fn normalize_file(file: &str) -> String {
	Path::new(file).components().collect::<PathBuf>().display().to_string()
}

fn source_url(file: &str, line: u32) -> String {
	let file = file.trim_start_matches("crates/estate/");
	let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), file);
	format!("file://{path}#{line}")
}
