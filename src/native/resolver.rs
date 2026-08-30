use crate::prelude::*;

use std::io::{Error, ErrorKind, Result};

pub fn home_dir() -> Result<PathBuf> {
	dirs::home_dir()
		.ok_or_else(|| Error::new(ErrorKind::NotFound, "Could not determine home directory"))
}

/// User global estate.
/// Example:
/// ~/.estate
pub(crate) fn global_estate_dir() -> Result<PathBuf> {
	Ok(home_dir()?.join(".estate"))
}

/// Current workspace/project root.
/// Example:
/// /Users/loi/projects/my-app
pub fn project_root() -> Result<PathBuf> {
	std::env::current_dir()
}

/// Project-local estate.
/// Example:
/// /Users/loi/projects/my-app/.estate
pub fn project_estate_dir() -> Result<PathBuf> {
	Ok(project_root()?.join(".estate"))
}

/// Legacy/project metadata directory.
/// Example:
/// ~/.leviticus
pub fn global_project_dir() -> Result<PathBuf> {
	Ok(home_dir()?.join(".leviticus"))
}

/// Estate engine application data.
/// Used for:
/// - indexes
/// - cache
/// - daemon state
/// - logs
pub fn engine_data_dir() -> Result<PathBuf> {
	dirs::data_dir()
		.map(|dir| dir.join("estate"))
		.ok_or_else(|| {
			Error::new(
				ErrorKind::NotFound,
				"Could not determine application data directory",
			)
		})
}

/// Estate engine cache.
/// Used for disposable generated data.
pub fn engine_cache_dir() -> Result<PathBuf> {
	dirs::cache_dir()
		.map(|dir| dir.join("estate"))
		.ok_or_else(|| Error::new(ErrorKind::NotFound, "Could not determine cache directory"))
}

pub fn path() -> Result<PathBuf> {
	let path = engine_data_dir()?.join("state.json");
	println!("STATE PATH: {}", path.display());
	Ok(path)
}
