use crate::{
	app::EstateState,
	native::{ resolver::*, daemon::daemon::*, prelude::*, * },
	prelude::*,
};

use std::{ fs, io::Result, path::{ Path, PathBuf } };

fn file_link(path: &Path) -> String {
	let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

	format!("file://{}", p.to_string_lossy())
}

fn explain_json_path() -> Result<PathBuf> {
	Ok(engine_data_dir()?.join("explain.json"))
}

fn manifest_json_path() -> Result<PathBuf> {
	Ok(engine_data_dir()?.join("manifest.json"))
}

fn symbols_json_path() -> Result<PathBuf> {
	Ok(engine_data_dir()?.join("symbols.json"))
}

fn read_symbols() -> serde_json::Value {
	let path = match symbols_json_path() {
		Ok(p) => p,
		Err(_) => {
			return serde_json::json!([]);
		}
	};

	let raw = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());

	serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::json!([]))
}

fn render_symbols(json: &serde_json::Value) -> String {
	let mut out = String::new();

	out.push_str("## Symbols (Clickable)\n\n");

	// -------------------------
	// FILE REGISTRY
	// -------------------------
	if
		let Some(fs) = json.get("filesystem") &&
		let Some(map) = fs.get("uid_mapping").and_then(|v| v.as_object())
	{
		out.push_str("### File Registry\n\n");

		let root = project_root().unwrap_or_else(|_| PathBuf::from("."));

		for (uid, path) in map {
			let rel = path.as_str().unwrap_or("");

			let full = root.join(rel);

			out.push_str(&format!("- `{}` → [{}]({})\n", uid, rel, file_link(&full)));
		}

		out.push('\n');
	}

	// -------------------------
	// ENGINE SYMBOLS
	// -------------------------
	if
		let Ok(path) = symbols_json_path() &&
		let Ok(raw) = fs::read_to_string(path) &&
		let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) &&
		let Some(arr) = json.get("symbols").and_then(|v| v.as_array())
	{
		out.push_str("### Estate Engine Symbols\n\n");

		for symbol in arr {
			let id = symbol
				.get("id")
				.and_then(|v| v.as_str())
				.unwrap_or("unknown");

			let path = symbol
				.get("path")
				.and_then(|v| v.as_str())
				.unwrap_or("");

			let full = PathBuf::from(path);

			out.push_str(
				&format!(
					"- `{}` → [{}]({}) — {}\n",
					id,
					path,
					file_link(&full),
					symbol
						.get("doc")
						.and_then(|v| v.as_str())
						.unwrap_or("")
				)
			);
		}

		out.push('\n');
	}

	out
}

pub fn generate_explain_doc() -> Result<()> {
	let workspace = project_root()?;

	EstateState::save_workspace(&workspace);

	let out = workspace.join("explain.md");

	let path = explain_json_path()?;

	if !path.exists() {
		println!("missing explain.json at {:?}", path);
		return Ok(());
	}

	let raw = fs::read_to_string(&path)?;

	let json: serde_json::Value = serde_json
		::from_str(&raw)
		.unwrap_or_else(|_| serde_json::json!({}));

	let md = render_explain(&json);

	println!("writing → {:?}", out);

	fs::write(out, md)?;

	Ok(())
}

pub fn generate_runtime_views() {
	println!("▶ generate_runtime_views CALLED");

	if let Err(err) = generate_explain_doc() {
		eprintln!("failed generating view: {err}");
	}
}

fn render_explain(json: &serde_json::Value) -> String {
	let mut out = String::new();

	out.push_str("# Estate Explain\n\n");

	// -------------------------
	// SUMMARY
	// -------------------------
	if let Some(summary) = json.get("summary") {
		out.push_str("## Summary\n\n");

		out.push_str(summary.as_str().unwrap_or("N/A"));

		out.push_str("\n\n");
	}

	// -------------------------
	// MANIFEST
	// -------------------------
	if let Some(manifest) = json.get("manifest") {
		out.push_str("## Manifest\n\n");

		if let Some(obj) = manifest.as_object() {
			for (k, v) in obj {
				out.push_str(&format!("- **{}**: {}\n", k, v));
			}
		}

		out.push('\n');
	}

	// -------------------------
	// SYMBOLS
	// -------------------------
	if let Some(symbols) = json.get("symbols") {
		out.push_str("## Registry\n\n");

		if let Some(arr) = symbols.as_array() {
			for symbol in arr {
				let id = symbol
					.get("id")
					.and_then(|v| v.as_str())
					.unwrap_or("unknown");

				let path = symbol
					.get("path")
					.and_then(|v| v.as_str())
					.unwrap_or("");

				out.push_str(&format!("- `{}` → `{}`\n", id, path));
			}
		}

		out.push('\n');
	}

	// -------------------------
	// RUNTIME STATE
	// -------------------------
	if let Some(state) = json.get("state") {
		out.push_str("## Runtime State\n\n");

		if let Some(obj) = state.as_object() {
			for (k, v) in obj {
				out.push_str(&format!("- **{}**: {}\n", k, v));
			}
		}

		out.push('\n');
	}

	out.push_str(&render_symbols(json));

	out
}

pub fn open_explain_doc() {
	todo!("open_explain_doc")
}
