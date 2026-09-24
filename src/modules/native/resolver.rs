use crate::prelude::*;

use anyhow::{Context, Result, anyhow};
// use std::{
// 	fs,
// 	path::{Path, PathBuf},
// };

pub fn crate_root() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn source_file(file: &str) -> PathBuf {
	crate_root().join(file)
}

pub fn filesystem_root(path: &Path) -> PathBuf {
	path.ancestors().last().unwrap().to_path_buf()
}
const PRECEDENCE: &[&str] = &["default", "profile", "project", "workspace"];

pub fn resolve_settings(file: impl AsRef<Path>, filename: &str) -> Result<Settings> {
	let walker = FsWalker::new_from_ref(file);

	let files = walker
		.find_named(filename)
		.with_context(|| format!("failed to find `{filename}`"))?;

	let mut layers: Vec<(String, serde_json::Map<String, serde_json::Value>)> = Vec::new();

	for path in files {
		let contents =
			fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

		let value: serde_json::Value = serde_json::from_str(&contents)
			.with_context(|| format!("failed to parse {}", path.display()))?;

		let object = value
			.as_object()
			.with_context(|| format!("{} must contain a JSON object", path.display()))?;

		let type_name = object
			.get("type")
			.and_then(serde_json::Value::as_str)
			.with_context(|| format!("{} is missing a string `type`", path.display()))?;

		if !PRECEDENCE.contains(&type_name) {
			anyhow::bail!("{} has unknown settings type `{type_name}`", path.display());
		}

		layers.push((type_name.to_owned(), object.clone()));
	}

	// Lowest → highest precedence.
	layers.sort_by_key(|(type_name, _)| {
		PRECEDENCE
			.iter()
			.position(|name| *name == type_name)
			.unwrap()
	});

	let mut resolved = serde_json::Map::new();

	for (_, settings) in layers {
		for (key, value) in settings {
			resolved.insert(key, value);
		}
	}

	let settings: Settings = serde_json::from_value(serde_json::Value::Object(resolved))?;

	Ok(settings)
}
pub fn ws_path() -> Result<PathBuf> {
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

	loop {
		let cargo_toml = path.join("Cargo.toml");

		if cargo_toml.is_file() {
			let contents = fs::read_to_string(&cargo_toml)?;

			if contents.contains("[workspace]") {
				return Ok(path);
			}
		}

		if !path.pop() {
			break;
		}
	}

	anyhow::bail!("could not find workspace root")
}
pub fn home_dir() -> Result<PathBuf> {
	dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))
}
pub fn workspace_cargo_path() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
pub fn global_estate_dir() -> Result<PathBuf> {
	Ok(home_dir()?.join(".estate"))
}
pub fn workspace_root() -> Result<PathBuf> {
	Ok(std::env::current_dir()?)
}
pub fn project_estate_dir() -> Result<PathBuf> {
	Ok(workspace_root()?.join(".estate"))
}
pub fn global_project_dir() -> Result<PathBuf> {
	Ok(home_dir()?.join(".leviticus"))
}
pub fn engine_data_dir() -> Result<PathBuf> {
	dirs::data_dir()
		.map(|dir| dir.join("estate"))
		.ok_or_else(|| anyhow!("Could not determine application data directory"))
}
pub fn engine_cache_dir() -> Result<PathBuf> {
	dirs::cache_dir()
		.map(|dir| dir.join("estate"))
		.ok_or_else(|| anyhow!("Could not determine application cache directory"))
}
pub fn engine_state_file() -> Result<PathBuf> {
	Ok(engine_data_dir()?.join("state.json"))
}
