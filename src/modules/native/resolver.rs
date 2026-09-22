use crate::prelude::*;

use anyhow::{Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};

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
		"could not locate Cargo workspace root from {}",
		env!("CARGO_MANIFEST_DIR")
	))
}

pub fn home_dir() -> Result<PathBuf> {
	dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))
}

pub fn workspace_cargo_path() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
/// User global estate.
/// Example:
/// ~/.estate
pub fn global_estate_dir() -> Result<PathBuf> {
	Ok(home_dir()?.join(".estate"))
}

/// Current workspace/project root.
/// Example:
/// /Users/loi/projects/my-app
pub fn workspace_root() -> Result<PathBuf> {
	Ok(std::env::current_dir()?)
}

/// Project-local estate.
/// Example:
/// /Users/loi/projects/my-app/.estate
pub fn project_estate_dir() -> Result<PathBuf> {
	Ok(workspace_root()?.join(".estate"))
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
		.ok_or_else(|| anyhow!("Could not determine application data directory"))
}
/// Estate engine cache.
/// Used for disposable generated data.
pub fn engine_cache_dir() -> Result<PathBuf> {
	dirs::cache_dir()
		.map(|dir| dir.join("estate"))
		.ok_or_else(|| anyhow!("Could not determine application cache directory"))
}

pub fn path() -> Result<PathBuf> {
	let path = engine_data_dir()?.join("state.json");
	Ok(path)
}

impl SpecialFiles {
	pub fn write(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = path.into();

		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		std::fs::write(path, contents)?;
		Ok(())
	}

	pub fn read_template(name: impl AsRef<Path>) -> Result<Option<String>> {
		let path = Self::ws_ai_template_dir()?.join(name);

		if !path.exists() {
			return Ok(None);
		}

		Ok(Some(std::fs::read_to_string(path)?))
	}
	pub fn ws_ai_template_dir() -> Result<PathBuf> {
		Ok(ws_path()?.join("ai").join("template"))
	}

	pub fn ws_log_dir() -> Result<PathBuf> {
		Ok(ws_path()?.join("log"))
	}

	pub fn ws_tmp_dir() -> Result<PathBuf> {
		Ok(Self::ws_log_dir()?.join("tmp"))
	}

	pub fn ws_sdlc_current_file() -> Result<PathBuf> {
		Ok(Self::ws_tmp_dir()?.join("sdlc.current.json"))
	}

	pub fn ws_sessions_dir() -> Result<PathBuf> {
		Ok(Self::ws_log_dir()?.join("session"))
	}

	pub fn ws_sessions_index() -> Result<PathBuf> {
		Ok(Self::ws_log_dir()?.join("sessions.json"))
	}

	pub fn ensure_dir(path: impl Into<PathBuf>) -> Result<PathBuf> {
		let path = path.into();
		fs::create_dir_all(&path)?;
		Ok(path)
	}

	pub fn write_json<T: Serialize>(path: impl Into<PathBuf>, value: &T) -> Result<()> {
		let path = path.into();

		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}

		let contents = serde_json::to_string_pretty(value)?;
		fs::write(path, contents)?;

		Ok(())
	}

	pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
		let contents = fs::read_to_string(path)?;
		Ok(serde_json::from_str(&contents)?)
	}
	pub fn append_text(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = path.into();

		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)?;
		}

		let mut file = std::fs::OpenOptions::new()
			.create(true)
			.append(true)
			.open(path)?;

		use std::io::Write;
		file.write_all(contents.as_ref())?;

		Ok(())
	}

	pub fn remove_file_if_exists(path: impl AsRef<Path>) -> Result<()> {
		let path = path.as_ref();

		if path.exists() {
			fs::remove_file(path)?;
		}

		Ok(())
	}

	pub fn ws_root() -> Result<PathBuf> {
		ws_path()
	}
}
impl SpecialFiles {
	pub fn ensure_parent(path: impl Into<PathBuf>) -> Result<PathBuf> {
		let path = path.into();

		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}

		Ok(path)
	}
	pub fn read_session_file(session: impl AsRef<Path>, name: impl AsRef<Path>) -> Result<String> {
		Ok(std::fs::read_to_string(Self::session_file(session, name))?)
	}
	pub fn read_intent(session: impl AsRef<Path>) -> Result<String> {
		Self::read_session_file(session, "intent.md")
	}
	pub fn read_spec(session: impl AsRef<Path>) -> Result<String> {
		Self::read_session_file(session, "spec.md")
	}
	pub fn read_tests(session: impl AsRef<Path>) -> Result<String> {
		Self::read_session_file(session, "tests.md")
	}
	pub fn read_verification(session: impl AsRef<Path>) -> Result<String> {
		Self::read_session_file(session, "verification.md")
	}
	pub fn read_plan(session: impl AsRef<Path>) -> Result<String> {
		Self::read_session_file(session, "plan.md")
	}
	pub fn read_progress(session: impl AsRef<Path>) -> Result<String> {
		Self::read_session_file(session, "progress.md")
	}

	pub fn session_file(session: impl AsRef<Path>, name: impl AsRef<Path>) -> PathBuf {
		session.as_ref().join(name)
	}
	pub fn session_intent(session: impl AsRef<Path>) -> PathBuf {
		Self::session_file(session, "intent.md")
	}
	pub fn session_spec(session: impl AsRef<Path>) -> PathBuf {
		Self::session_file(session, "spec.md")
	}
	pub fn session_tests(session: impl AsRef<Path>) -> PathBuf {
		Self::session_file(session, "tests.md")
	}
	pub fn session_plan(session: impl AsRef<Path>) -> PathBuf {
		Self::session_file(session, "plan.md")
	}
	pub fn session_progress(session: impl AsRef<Path>) -> PathBuf {
		Self::session_file(session, "progress.md")
	}
}
pub struct SpecialFiles;
