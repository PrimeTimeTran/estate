use crate::prelude::*;

use anyhow::{Context, Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};
use std::{
	fs,
	io::Write,
	path::{Path, PathBuf},
};

pub fn filesystem_root(path: &Path) -> PathBuf {
	path.ancestors().last().unwrap().to_path_buf()
}
pub fn crate_root() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

	Err(anyhow!(
		"could not locate workspace root from {}",
		env!("CARGO_MANIFEST_DIR")
	))
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
