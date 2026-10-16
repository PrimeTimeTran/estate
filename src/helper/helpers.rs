use crate::prelude::*;

use owo_colors::OwoColorize;
use terminal_size::{Width, terminal_size};

pub fn print(level: Level, message: impl Display) {
	println!("{} {}", level.format(level.label()), message,);
}

pub fn print_debug<T: Debug>(level: Level, name: &str, value: &T) {
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

pub fn print_section_info(
	title: &str,
	file: &str,
	line: u32,
	metrics: &[(&str, String)],
	values: &[(&str, String)],
) {
	let file = normalize_file(file);
	let width = terminal_size()
		.map(|(Width(width), _)| width as usize)
		.unwrap_or(80);

	let rule = "-".repeat(width);

	println!("{}", rule.dimmed());

	// Build the title-row adornment from all metrics.
	let metric_text = metrics
		.iter()
		.map(|(key, value)| format!("{key} {value}"))
		.collect::<Vec<_>>()
		.join(" │ ");

	let title_width = title.chars().count();
	let metric_width = metric_text.chars().count();
	let gap = width.saturating_sub(title_width + metric_width).max(1);

	print!("{}", title.bold().cyan());
	print!("{}", " ".repeat(gap));

	for (index, (key, value)) in metrics.iter().enumerate() {
		if index > 0 {
			print!(" {}", "│".dimmed());
			print!(" ");
		}

		print!("{} {}", key.dimmed(), value.yellow());
	}

	println!();

	// Detail rows, if supplied.
	if !values.is_empty() {
		let key_width = values
			.iter()
			.map(|(key, _)| key.chars().count())
			.max()
			.unwrap_or(0);

		for (key, value) in values {
			print!("  {:<key_width$}  ", key.dimmed());
			println!("{value}");
		}
	}

	println!(
		"{}",
		format!(
			"See {}",
			hyperlink(&format!("{file}#{line}"), &source_url(&file, line))
		)
		.dimmed()
	);

	println!("{}", rule.dimmed());
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
	pub fn format(self, text: impl Display) -> String {
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

pub fn success<T: Display>(value: T) -> String {
	value.green().to_string()
}

pub fn error<T: Display>(value: T) -> String {
	value.red().to_string()
}

pub fn warning<T: Display>(value: T) -> String {
	value.yellow().to_string()
}

pub fn info<T: Display>(value: T) -> String {
	value.cyan().to_string()
}

pub fn debug<T: Display>(value: T) -> String {
	value.magenta().to_string()
}

pub fn muted<T: Display>(value: T) -> String {
	value.dimmed().to_string()
}

pub fn bold<T: Display>(value: T) -> String {
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
