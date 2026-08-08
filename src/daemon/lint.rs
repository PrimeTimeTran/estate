use std::{
	collections::HashMap,
	fs,
	io::{self, BufRead, BufReader},
	ops::Range,
	path::{Path, PathBuf},
};
// I want to re architecture this as a compiler actually. Can u help me with that?
// 1. check file and determine "block comment prefix.
// 2. find the block comment ranges.
// 3. expand block comment beginning and end when necessary.
// 4. check gutter /align column for "block comment prefix" + 7 columns (space for auto white space and  6 "#"s for all .md depths.
// 5. Shift chars right if they're inside of those column indexes(excluding # tags)
// 6. Check the chars remaining, if they're hashing, then shift them left(because I might have edited the row/comment after a previous edit
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
///      (3) Paths
// pub fn run<P: AsRef<Path>>(&self, path: P) -> io::Result<()>
// pub fn remove_blank_lines(path: impl AsRef<Path>) -> io::Result<()>
// pub fn collapse_empty_lines(file_path: &PathBuf) -> io::Result<()>
///--------------------------------------------------------------------------------
///      [1]
///      Most generic and reusable.
///      pub fn run<P: AsRef<Path>>(&self, path: P) -> io::Result<()>
///
///      `P` can be any type that can be viewed as a `Path`.
///
///      Accepts:
///      - PathBuf
///      - &Path
///      - &PathBuf
///      - &str
///      - String
///
///      This is the idiomatic choice for public APIs because callers don't
///      have to convert their path into a specific type first.
///--------------------------------------------------------------------------------
///      [2]
///      pub fn remove_blank_lines(path: impl AsRef<Path>) -> io::Result<()>
///      Same capability as the generic version above.
///
///      `impl AsRef<Path>` is simply shorthand for:
///
///  <P: AsRef<Path>>
///
///      These compile to essentially the same thing. Use this when the type
///      parameter doesn't need to be named anywhere else in the signature.
///
///      Many people find this version easier to read.
///--------------------------------------------------------------------------------
///      pub fn collapse_empty_lines(file_path: &PathBuf) -> io::Result<()>
///      Least flexible.
///
///      Only accepts a borrowed `PathBuf`.
///
///      Does NOT directly accept:
///      - &Path
///      - &str
///      - String
///
///      The caller must already have a `PathBuf`, making this API more
///      restrictive than necessary.
///
///      Prefer `&Path` if you only need a borrowed path.
///      Why choose one over another?
///      AsRef<Path>
///      impl AsRef<Path>
///      &PathBuf
///
// | Signature          | Flexibility | Idiomatic?      | When to use                                      |
// | ------------------ | ----------- | --------------- | ------------------------------------------------ |
// | `P: AsRef<Path>`   | ⭐⭐⭐⭐⭐       | ✅               | Public APIs, libraries                           |
// | `impl AsRef<Path>` | ⭐⭐⭐⭐⭐       | ✅               | Same as above, cleaner syntax                    |
// | `&Path`            | ⭐⭐⭐⭐        | ✅               | Borrowing only, no ownership needed              |
// | `&PathBuf`         | ⭐⭐          | ❌ Usually avoid | Only if you specifically need a `PathBuf` (rare) |
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
		// Iterate through each line
		for line_result in reader.lines() {
			let line = line_result?;
			// Check if the line is anything other than empty or just spaces/tabs
			if !line.chars().all(|c| c.is_whitespace()) {
				filtered_lines.push(line);
			}
		}
		// Join all surviving lines back together with a single newline character
		let new_content = filtered_lines.join("\n");
		// Overwrite the original file with the modified content
		// (You can add a trailing newline if desired: format!("{}\n", new_content))
		fs::write(file_path, new_content)?;
		Ok(())
	}
	// pub fn new(width: usize) -> Self {
	//     let mut compiler = Self {
	//         width,
	//         configs: HashMap::new(),
	//     };
	//     // Register default language profiles
	//     compiler.register_language("rs", "///", "///-");
	//     compiler.register_language("lua", "--", "---");
	//     compiler.register_language("sql", "--", "---");
	//     compiler
	// }
	/// Register a custom language prefix configuration
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
				Some(CommentStyle::RustDoc)
			} else if trimmed.starts_with("//!") {
				Some(CommentStyle::RustInner)
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
	// Step 3, 4, 5, & 6: Transform lines (Expansion, Gutter alignment, Shift right/left idempotency)
	// pub fn compile_source(&self, input: &str) -> String {
	//     let lines: Vec<String> = input.lines().map(String::from).collect();
	//     let rule_line = format!("///{}", "-".repeat(self.width));
	//     let mut output_lines = Vec::with_capacity(lines.len());
	//     for line in &lines {
	//         // Step 3: Expand exact rules
	//         if line == "///-" {
	//             output_lines.push(rule_line.clone());
	//             continue;
	//         }
	//         // Preserve existing valid full-line dividers
	//         if line.starts_with("///-") && line.chars().all(|c| c == '/' || c == '-') {
	//             output_lines.push(line.clone());
	//             continue;
	//         }
	//         // Process doc comments
	//         if line.starts_with("///") {
	//             output_lines.push(self.transform_doc_line(line));
	//         } else {
	//             output_lines.push(line.clone());
	//         }
	//     }
	//     output_lines.join("\n") + if input.ends_with('\n') { "\n" } else { "" }
	// }
	// fn transform_doc_line(&self, line: &str) -> String {
	//     let prefix = "///";
	//     let body = &line[prefix.len()..];
	//     // If the line is empty after prefix, keep it clean
	//     if body.is_empty() {
	//         return line.to_string();
	//     }
	//     // Step 4 & 5 & 6: Parse headers vs regular text with gutter awareness & idempotency
	//     let trimmed_body = body.trim_start();
	//     if trimmed_body.starts_with('#') {
	//         // Markdown header handling (up to 6 hashes)
	//         let hashes: String = trimmed_body.chars().take_while(|&c| c == '#').collect();
	//         let depth = hashes.len();
	//         if depth <= 6 {
	//             let text_after_hashes = trimmed_body[depth..].trim_start();
	//             // 7 columns allocated for header depth formatting (e.g. pad = 7 - depth)
	//             let pad = 7_usize.saturating_sub(depth);
	//             return format!("{}{}{}{}", prefix, hashes, " ".repeat(pad), text_after_hashes);
	//         }
	//     }
	//     // Normal text column check / shift logic (Idempotent left/right adjustment)
	//     // Find leading spaces of the body relative to column index
	//     let leading_spaces = body.chars().take_while(|&c| c == ' ').count();
	//     // Absolute column tracking: prefix takes 3 columns, so body starts at column 4 (1-based index)
	//     // Gutter alignment target: "prefix" (3 cols) + 7 reserved columns for depth/alignment = column 10 threshold.
	//     let current_content_pos = leading_spaces + 3;
	//     let clean_content = body.trim_start();
	//     if clean_content.is_empty() {
	//         return line.to_string();
	//     }
	//     // If it's already aligned or pushed past the target gutter, normalize it back (Step 6: Shift left if previously over-edited)
	//     // Or shift right if it falls before column 10 (Step 5)
	//     let target_col = 10_usize;
	//     if current_content_pos < target_col {
	//         let padding_needed = target_col - current_content_pos;
	//         format!("{}{}{}", prefix, " ".repeat(padding_needed), clean_content)
	//     } else {
	//         // Already past or at target, normalize leading space to avoid runaway growth
	//         format!("{}{}", prefix, clean_content)
	//     }
	// }
}
// const WIDTH: usize = 80;
// fn align_doc(line: &str) -> String {
//     if !line.starts_with("///") {
//         return line.to_string();
//     }
//     let body = &line[3..];
//     let clean = body.trim_start();
//     if clean.starts_with('#') {
//         let hashes: String = clean.chars().take_while(|&c| c == '#').collect();
//         let depth = hashes.len();
//         let text = clean[depth..].trim_start();
//         let pad = 7_usize.saturating_sub(depth);
//         return format!("///{}{}{}", hashes, " ".repeat(pad), text);
//     }
//     // Normal text alignment
//     if body.is_empty() {
//         return line.to_string();
//     }
//     let leading_spaces = body.chars().take_while(|&c| c == ' ').count();
//     let pos = leading_spaces + 3; // +2 for /// prefix offset adjustment relative to awk logic
//     if pos >= 10 {
//         return line.to_string();
//     }
//     let padding_needed = 10 - pos;
//     format!("///{}{}", " ".repeat(padding_needed), body)
// }
// fn process_content<R: BufRead, W: Write>(reader: R, mut writer: W) -> io::Result<()> {
//     let rule = format!("///{}", "-".repeat(WIDTH));
//     for line_res in reader.lines() {
//         let line = line_res?;
//         if line == "///-" {
//             writeln!(writer, "{}", rule)?;
//         } else if line.starts_with("///-") && line.chars().all(|c| c == '/' || c == '-') {
//             writeln!(writer, "{}", line)?;
//         } else if line.starts_with("///") {
//             writeln!(writer, "{}", align_doc(&line))?;
//         } else {
//             writeln!(writer, "{}", line)?;
//         }
//     }
//     Ok(())
// }
#[derive(Debug, Clone)]
pub struct LanguageConfig {
	pub prefix: String,
	pub rule_prefix: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentStyle {
	RustDoc,   // ///
	RustInner, // //!
}
impl CommentStyle {
	pub fn prefix(&self) -> &'static str {
		match self {
			CommentStyle::RustDoc => "///",
			CommentStyle::RustInner => "//!",
		}
	}
}
#[derive(Debug, Clone)]
pub struct CommentBlock {
	pub line_range: Range<usize>,
	pub style: CommentStyle,
}
