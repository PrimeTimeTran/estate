use std::path::PathBuf;

pub fn home_dir() -> std::io::Result<PathBuf> {
	dirs::home_dir().ok_or_else(|| {
		std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"Could not determine home directory",
		)
	})
}

/// User global estate.
/// Example:
/// ~/.estate
pub fn global_estate_dir() -> std::io::Result<PathBuf> {
	Ok(home_dir()?.join(".estate"))
}

/// Current workspace/project root.
/// Example:
/// /Users/loi/projects/my-app
pub fn project_root() -> std::io::Result<PathBuf> {
	std::env::current_dir()
}

/// Project-local estate.
/// Example:
/// /Users/loi/projects/my-app/.estate
pub fn project_estate_dir() -> std::io::Result<PathBuf> {
	Ok(project_root()?.join(".estate"))
}

/// Legacy/project metadata directory.
/// Example:
/// ~/.leviticus
pub fn global_project_dir() -> std::io::Result<PathBuf> {
	Ok(home_dir()?.join(".leviticus"))
}

/// Estate engine application data.
/// Used for:
/// - indexes
/// - cache
/// - daemon state
/// - logs
pub fn engine_data_dir() -> std::io::Result<PathBuf> {
	dirs::data_dir()
		.map(|dir| dir.join("estate"))
		.ok_or_else(|| {
			std::io::Error::new(
				std::io::ErrorKind::NotFound,
				"Could not determine application data directory",
			)
		})
}

/// Estate engine cache.
/// Used for disposable generated data.
pub fn engine_cache_dir() -> std::io::Result<PathBuf> {
	dirs::cache_dir()
		.map(|dir| dir.join("estate"))
		.ok_or_else(|| {
			std::io::Error::new(
				std::io::ErrorKind::NotFound,
				"Could not determine cache directory",
			)
		})
}
