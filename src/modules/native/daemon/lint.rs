use crate::prelude::*;

use std::io::{self, BufRead, BufReader};

pub struct DocCompiler {
	width: usize,
	configs: HashMap<String, LanguageConfig>,
}

impl Default for DocCompiler {
	fn default() -> Self {
		Self {
			width: 80,
			configs: HashMap::new(),
		}
	}
}
impl DocCompiler {
	pub fn new() -> Self {
		Self::default()
	}
}
///--------------------------------------------------------------------------------
/// # (3) Paths
/// Three pseudostring type argumnents. When to use each given their different ownership models.
///
/// pub fn run<P: AsRef<Path>>(&self, path: P) -> io::Result<()>
/// pub fn remove_blank_lines(path: impl AsRef<Path>) -> io::Result<()>
/// pub fn collapse_empty_lines(file_path: &PathBuf) -> io::Result<()>
///
/// ## [1] pub fn run<P: AsRef<Path>>(&self, path: P) -> io::Result<()>
///   Most generic and reusable.
///   `P` can be any type that can be viewed as a `Path`.
///   Accepts:
///   - PathBuf
///   - &Path
///   - &PathBuf
///   - &str
///   - String
///   This is the idiomatic choice for public APIs because callers don't
///   have to convert their path into a specific type first.
/// --------------------------------------------------------------------------------
/// ## [2] remove_blank_lines(path: impl AsRef<Path>) -> io::Result<()>
///   Same capability as the generic version above.
///   `impl AsRef<Path>` is simply shorthand for:
/// <P: AsRef<Path>>
///   These compile to essentially the same thing. Use this when the type
///   parameter doesn't need to be named anywhere else in the signature.
///   Many people find this version easier to read.
/// --------------------------------------------------------------------------------
///   ## [3] pub fn collapse_empty_lines(file_path: &PathBuf) -> io::Result<()>
///   Least flexible. Only accepts a borrowed `PathBuf`.
///   Does NOT directly accept:
///   - &Path
///   - &str
///   - String
///   The caller must already have a `PathBuf`, making this API more
///   restrictive than necessary.
///   Prefer `&Path` if you only need a borrowed path.
///   Why choose one over another?
///   AsRef<Path>
///   impl AsRef<Path>
///   &PathBuf
///
/// | Signature          | Flexibility | Idiomatic?       | When to use                                      |
/// | ------------------ | ----------- | ---------------- | ------------------------------------------------ |
/// | `P: AsRef<Path>`   | ⭐⭐⭐⭐⭐  | ✅               | Public APIs, libraries                           |
/// | `impl AsRef<Path>` | ⭐⭐⭐⭐⭐  | ✅               | Same as above, cleaner syntax                    |
/// | `&Path`            | ⭐⭐⭐⭐    | ✅               | Borrowing only, no ownership needed              |
/// | `&PathBuf`         | ⭐⭐        | ❌ Usually avoid | Only if you specifically need a `PathBuf` (rare) |
impl DocCompiler {
	pub fn remove_blank_lines<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
		// use std::process::Command;
		// Command::new("cargo")
		//     .args(["fmt", "--"])
		//     .arg(path)
		//     .status()?;
		let path = path.as_ref();
		// Step 1: Look up config for this file format before running compilation
		let config = self.resolve_config(path);
		let content = fs::read_to_string(path)?;
		let compiled = self.compile_source(&content, &config);
		fs::write(path, compiled)?;
		Ok(())
	}
	pub fn run(&self, path: impl AsRef<Path>) -> io::Result<()> {
		let contents = fs::read_to_string(&path)?;
		let output = contents
			.lines()
			.filter(|line| !line.trim().is_empty())
			.collect::<Vec<_>>()
			.join("\n");
		fs::write(&path, output)?;
		self.remove_blank_lines(path);
		Ok(())
	}
	/// Removes all empty or whitespace-only lines from a Rust source file,
	/// causing all remaining lines (including comments) to "touch" each other.
	pub fn collapse_empty_lines(file_path: &PathBuf) -> io::Result<()> {
		// Open the file for reading
		let file = fs::File::open(file_path)?;
		let reader = BufReader::new(file);
		let mut filtered_lines = Vec::new();
		for line_result in reader.lines() {
			let line = line_result?;
			if !line.chars().all(|c| c.is_whitespace()) {
				filtered_lines.push(line);
			}
		}
		// Join all surviving lines back together with a single newline character
		let new_content = filtered_lines.join("\n");
		fs::write(file_path, new_content)?;
		Ok(())
	}
	pub fn register_language(&mut self, ext: &str, prefix: &str, rule_prefix: &str) {
		self.configs.insert(
			ext.to_lowercase(),
			LanguageConfig {
				prefix: prefix.to_string(),
				rule_prefix: rule_prefix.to_string(),
			},
		);
	}
	/// Stage 1: Resolve the configuration for the target file based on its extension
	pub fn resolve_config<P: AsRef<Path>>(&self, path: P) -> LanguageConfig {
		let ext = path
			.as_ref()
			.extension()
			.and_then(|e| e.to_str())
			.unwrap_or("rs")
			.to_lowercase();
		// Fallback to Rust style (`///`) if extension isn't found in the map
		self
			.configs
			.get(&ext)
			.cloned()
			.unwrap_or_else(|| LanguageConfig {
				prefix: "///".to_string(),
				rule_prefix: "///-".to_string(),
			})
	}
	/// Step 1 & 2: Check file, determine prefix, and find contiguous block comment ranges.
	pub fn find_comment_blocks(&self, lines: &[String]) -> Vec<CommentBlock> {
		let mut blocks = Vec::new();
		let mut start_idx: Option<usize> = None;
		let mut current_style: Option<CommentStyle> = None;
		for (i, line) in lines.iter().enumerate() {
			let trimmed = line.trim_start();
			let style = if trimmed.starts_with("///") {
				Some(CommentStyle::DocOuter)
			} else if trimmed.starts_with("//!") {
				Some(CommentStyle::DocInner)
			} else {
				None
			};
			match (current_style, style) {
				(None, Some(s)) => {
					// Start of a new block
					start_idx = Some(i);
					current_style = Some(s);
				}
				(Some(curr_s), Some(s)) if curr_s == s => {
					// Continuing current block
				}
				(Some(_), _) => {
					// End of block
					if let Some(start) = start_idx {
						blocks.push(CommentBlock {
							line_range: start..i,
							style: current_style.unwrap(),
						});
					}
					start_idx = None;
					current_style = None;
					// Check if this current line is the start of a *new* block
					if let Some(s) = style {
						start_idx = Some(i);
						current_style = Some(s);
					}
				}
				(None, None) => {}
			}
		}
		// Catch trailing block
		if let Some(start) = start_idx {
			blocks.push(CommentBlock {
				line_range: start..lines.len(),
				style: current_style.unwrap(),
			});
		}
		blocks
	}
	pub fn compile_source(&self, input: &str, config: &LanguageConfig) -> String {
		let lines: Vec<String> = input.lines().map(String::from).collect();
		let rule_line = format!("{}{}", config.prefix, "-".repeat(self.width));
		let mut output_lines = Vec::with_capacity(lines.len());
		for line in &lines {
			// Expand dynamic rule match (e.g. ///- or ---)
			if line == &config.rule_prefix {
				output_lines.push(rule_line.clone());
				continue;
			}
			// Preserve existing valid full-line dividers
			if line.starts_with(&config.rule_prefix)
				&& line
					.chars()
					.all(|c| c == config.prefix.chars().next().unwrap() || c == '-')
			{
				output_lines.push(line.clone());
				continue;
			}
			// Process comments matching the resolved prefix
			if line.starts_with(&config.prefix) {
				output_lines.push(self.transform_doc_line(line, &config.prefix));
			} else {
				output_lines.push(line.clone());
			}
		}
		output_lines.join("\n") + if input.ends_with('\n') { "\n" } else { "" }
	}
	fn transform_doc_line(&self, line: &str, prefix: &str) -> String {
		let body = &line[prefix.len()..];
		if body.is_empty() {
			return line.to_string();
		}
		let trimmed_body = body.trim_start();
		if trimmed_body.starts_with('#') {
			let hashes: String = trimmed_body.chars().take_while(|&c| c == '#').collect();
			let depth = hashes.len();
			if depth <= 6 {
				let text_after_hashes = trimmed_body[depth..].trim_start();
				let pad = 7_usize.saturating_sub(depth);
				return format!(
					"{}{}{}{}",
					prefix,
					hashes,
					" ".repeat(pad),
					text_after_hashes
				);
			}
		}
		let leading_spaces = body.chars().take_while(|&c| c == ' ').count();
		let current_content_pos = leading_spaces + prefix.len();
		let clean_content = body.trim_start();
		if clean_content.is_empty() {
			return line.to_string();
		}
		let target_col = 10_usize;
		if current_content_pos < target_col {
			let padding_needed = target_col - current_content_pos;
			format!("{}{}{}", prefix, " ".repeat(padding_needed), clean_content)
		} else {
			format!("{}{}", prefix, clean_content)
		}
	}
}
#[derive(Debug, Clone)]
pub struct LanguageConfig {
	pub prefix: String,
	pub rule_prefix: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
	Line,          // //
	Block,         // /*
	DocOuter,      // ///
	DocInner,      // //!
	DocBlock,      // /**
	DocInnerBlock, // /*!
}
impl CommentStyle {
	pub fn prefix(&self) -> &'static str {
		match self {
			CommentStyle::Line => "//",
			CommentStyle::Block => "/*",
			CommentStyle::DocOuter => "///",
			CommentStyle::DocInner => "//!",
			CommentStyle::DocBlock => "//**",
			CommentStyle::DocInnerBlock => "/*!",
			_ => {
				todo!("go")
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct CommentBlock {
	pub line_range: std::ops::Range<usize>,
	pub style: CommentStyle,
}
