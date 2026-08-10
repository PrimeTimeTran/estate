#![allow(warnings)]

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// ==========================================
// 1. DATA STRUCTURES & MANIFESTS
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SectionConfig {
	#[serde(default = "default_path")]
	pub path: String,
	#[serde(default)]
	pub description: Option<String>,
}

fn default_path() -> String {
	"./".to_string()
}

pub struct EstateSection {
	pub config: SectionConfig,
	pub items: Vec<String>,
}

// ==========================================
// 2. PARSER ENGINE FOR ESTATE.MD
pub fn parse_estate_markdown(content: &str) -> Vec<EstateSection> {
	let mut sections = Vec::new();
	let mut current_config = SectionConfig {
		path: "./".to_string(),
		description: None,
	};
	let mut current_items = Vec::new();
	let mut in_loi_block = false;
	let mut in_manifest_list = false;
	let mut loi_buffer = String::new();
	for line in content.lines() {
		let trimmed = line.trim();
		if trimmed.starts_with("# ") || trimmed.starts_with("## ") {
			if !current_items.is_empty() || current_config.path != "./" {
				sections.push(EstateSection {
					config: current_config.clone(),
					items: current_items,
				});
				current_items = Vec::new();
			}
			in_manifest_list = false;
		}
		if trimmed.starts_with("```loi") {
			in_loi_block = true;
			in_manifest_list = false;
			loi_buffer.clear();
			continue;
		} else if in_loi_block && trimmed.starts_with("```") {
			in_loi_block = false;
			in_manifest_list = true;
			current_config = parse_loi_frontmatter(&loi_buffer);
			continue;
		}
		if in_loi_block {
			loi_buffer.push_str(line);
			loi_buffer.push('\n');
			continue;
		}
		if in_manifest_list && (trimmed.starts_with('-') || trimmed.starts_with('*')) {
			let raw_item = trimmed
				.trim_start_matches('-')
				.trim_start_matches('*')
				.trim();
			let cleaned_item = raw_item.trim_matches('"').trim_matches('\'').to_string();
			if !cleaned_item.is_empty() {
				current_items.push(cleaned_item);
			}
		}
	}
	if !current_items.is_empty() {
		sections.push(EstateSection {
			config: current_config,
			items: current_items,
		});
	}
	sections
}
fn parse_loi_frontmatter(buffer: &str) -> SectionConfig {
	let mut path = "./".to_string();
	let mut desc_lines = Vec::new();
	let mut capturing_desc = false;
	for line in buffer.lines() {
		let trimmed = line.trim();
		if trimmed.starts_with("path:") {
			capturing_desc = false;
			path = trimmed.strip_prefix("path:").unwrap().trim().to_string();
		} else if trimmed.starts_with("description:") {
			capturing_desc = true;
			let initial_desc = trimmed.strip_prefix("description:").unwrap().trim();
			if !initial_desc.is_empty() {
				desc_lines.push(initial_desc.to_string());
			}
		} else if capturing_desc {
			if trimmed.starts_with('-') {
				desc_lines.push(trimmed.trim_start_matches('-').trim().to_string());
			} else {
				desc_lines.push(trimmed.to_string());
			}
		}
	}
	SectionConfig {
		path,
		description: if desc_lines.is_empty() {
			None
		} else {
			Some(desc_lines.join("\n"))
		},
	}
}

// ==========================================
// 3. PAYLOAD PROJECTION ENGINE
fn resolve_target_base(path_expr: &str) -> PathBuf {
	if path_expr.starts_with("~/") || path_expr == "~" {
		let home_dir = env::var("HOME")
			.or_else(|_| env::var("USERPROFILE"))
			.unwrap_or_else(|_| ".".to_string());
		if path_expr == "~" {
			PathBuf::from(home_dir)
		} else {
			Path::new(&home_dir).join(&path_expr[2..])
		}
	} else {
		env::current_dir()
			.unwrap_or_else(|_| PathBuf::from("."))
			.join(path_expr)
	}
}

/// Materializes authored assets. In Phase 2, this can pull from pre-written template source payloads
/// rather than just generating default empty stubs.
pub fn materialize_payload(estate_file_path: &Path) -> std::io::Result<()> {
	println!(
		"📦 Reading payload manifest from: {}",
		estate_file_path.display()
	);
	if !estate_file_path.exists() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			format!(
				"Payload manifest not found at {}",
				estate_file_path.display()
			),
		));
	}
	let content = fs::read_to_string(estate_file_path)?;
	let sections = parse_estate_markdown(&content);
	for section in sections {
		let base_target = resolve_target_base(&section.config.path);
		println!(
			"\n📂 Payload Scope: [{}] -> Resolved: {}",
			section.config.path,
			base_target.display()
		);
		if let Some(ref desc) = section.config.description {
			println!("   ℹ️ {}", desc.lines().next().unwrap_or(""));
		}
		for item in section.items {
			let is_dir = item.ends_with('/') || !item.contains('.');
			let clean_item = item.trim_end_matches('/');
			let target_path = base_target.join(clean_item);
			if is_dir {
				fs::create_dir_all(&target_path)?;
				println!("     [+] Seeded Directory:     {}", target_path.display());
			} else {
				if let Some(parent) = target_path.parent() {
					fs::create_dir_all(parent)?;
				}
				// Phase 2 behavior: If a pre-authored source payload exists in mold assets, copy it over.
				// Otherwise fallback to a structured authored default.
				if !target_path.exists() {
					let authored_content = format!(
						"// ==========================================\n\
                         // Authored Payload Target: {}\n\
                         // Generated via DIP Pipeline Stage 2\n\
                         // ==========================================\n\n\
                         pub fn entry() {{\n    println!(\"Loaded payload component successfully.\");\n}}\n",
						clean_item
					);
					fs::write(&target_path, authored_content)?;
					println!("     [+] Projected Payload:    {}", target_path.display());
				} else {
					println!("     [=] Preserved Production: {}", target_path.display());
				}
			}
		}
	}
	println!("\n✨ Estate payload generation completed successfully.");
	Ok(())
}

// ==========================================
// 4. CLI BINARY ENTRYPOINT
fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	// Pointing to your stage 2 mold target layout
	let default_manifest = Path::new("src/mold/2-gen-estate-from-filled-persona.md");
	let manifest_path = args
		.windows(2)
		.find(|w| (w[0] == "-m" || w[0] == "--manifest"))
		.map(|w| PathBuf::from(&w[1]))
		.unwrap_or_else(|| default_manifest.to_path_buf());
	println!("⚡ Initializing Delta Integral Paradigm (DIP) Payload Generator...");
	materialize_payload(&manifest_path)?;
	Ok(())
}
