use crate::prelude::*;

use owo_colors::OwoColorize;

pub fn print(level: Level, message: impl std::fmt::Display) {
	println!("{} {}", level.format(level.label()), message,);
}

pub fn print_debug<T: std::fmt::Debug>(level: Level, name: &str, value: &T) {
	println!("{} {} = {:#?}", level.format(level.label()), name, value,);
}

pub fn print_section(title: &str, file: &str, line: u32) {
	let file = normalize_file(file);
	println!("{}", "-".repeat(80).dimmed());
	println!("{}", title.bold().cyan());
	println!(
		"See {}",
		hyperlink(&format!("{file}#{line}"), &source_url(&file, line)).dimmed()
	);
	println!("{}", "-".repeat(80).dimmed());
}

/// Prints a section and then panics.
///
/// The panic is deliberately not caught, so Rust's normal panic
/// handling, location reporting, and backtrace remain intact.
pub fn panic_section(title: &str, message: &str, file: &str, line: u32) -> ! {
	print_section(title, file, line);
	panic!("{message}");
}

pub fn hyperlink(text: &str, url: &str) -> String {
	format!("\x1b]8;;{url}\x07{text}\x1b]8;;\x07")
}

fn normalize_file(file: &str) -> String {
	Path::new(file)
		.components()
		.collect::<PathBuf>()
		.display()
		.to_string()
}

fn source_url(file: &str, line: u32) -> String {
	let file = file.trim_start_matches("crates/estate/");
	let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), file);
	format!("file://{path}#{line}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
	Trace,
	Debug,
	Info,
	Success,
	Warn,
	Error,
}
impl Level {
	pub fn label(self) -> &'static str {
		match self {
			Self::Trace => "TRACE",
			Self::Debug => "DEBUG",
			Self::Info => "INFO",
			Self::Success => "OK",
			Self::Warn => "WARN",
			Self::Error => "ERROR",
		}
	}
}

impl Level {
	pub fn format(self, text: impl std::fmt::Display) -> String {
		match self {
			Self::Trace => text.dimmed().to_string(),
			Self::Debug => text.magenta().to_string(),
			Self::Info => text.cyan().to_string(),
			Self::Success => text.green().to_string(),
			Self::Warn => text.yellow().to_string(),
			Self::Error => text.red().to_string(),
		}
	}
}

pub fn success<T: std::fmt::Display>(value: T) -> String {
	value.green().to_string()
}

pub fn error<T: std::fmt::Display>(value: T) -> String {
	value.red().to_string()
}

pub fn warning<T: std::fmt::Display>(value: T) -> String {
	value.yellow().to_string()
}

pub fn info<T: std::fmt::Display>(value: T) -> String {
	value.cyan().to_string()
}

pub fn debug<T: std::fmt::Display>(value: T) -> String {
	value.magenta().to_string()
}

pub fn muted<T: std::fmt::Display>(value: T) -> String {
	value.dimmed().to_string()
}

pub fn bold<T: std::fmt::Display>(value: T) -> String {
	value.bold().to_string()
}

pub fn success_label() -> String {
	"SUCCESS".green().bold().to_string()
}

pub fn error_label() -> String {
	"ERROR".red().bold().to_string()
}

pub fn warning_label() -> String {
	"WARN".yellow().bold().to_string()
}

pub fn info_label() -> String {
	"INFO".cyan().bold().to_string()
}

pub fn debug_label() -> String {
	"DEBUG".magenta().bold().to_string()
}

pub fn divider() {
	println!("{}", "-".repeat(80).dimmed());
}

// pub fn title(text: impl Display) {
// 	println!("{}", text.bold().cyan());
// }

// pub fn label(text: impl Display) -> String {
// 	text.bold().to_string()
// }

// pub fn muted(text: impl Display) -> String {
// 	text.dimmed().to_string()
// }

// pub fn success(text: impl Display) -> String {
// 	text.green().to_string()
// }

// pub fn warning(text: impl Display) -> String {
// 	text.yellow().to_string()
// }

// pub fn error(text: impl Display) -> String {
// 	text.red().to_string()
// }

// pub fn info(text: impl Display) -> String {
// 	text.cyan().to_string()
// }
