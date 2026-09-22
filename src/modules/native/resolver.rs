use crate::prelude::*;

use anyhow::{Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};
use std::{
	fs,
	io::Write,
	path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy)]
pub enum SessionFile {
	Intent,
	Spec,
	Tests,
	Verification,
	Plan,
	Progress,
}

#[derive(Debug, Clone, Copy)]
pub enum SpecialFile {
	AiTemplateDir,
	LogDir,
	TmpDir,
	SdlcCurrent,
	SessionsDir,
	SessionsIndex,
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

impl SessionFile {
	pub fn name(self) -> &'static str {
		match self {
			Self::Intent => "intent.md",
			Self::Spec => "spec.md",
			Self::Tests => "tests.md",
			Self::Verification => "verification.md",
			Self::Plan => "plan.md",
			Self::Progress => "progress.md",
		}
	}

	pub fn path(self, session: impl AsRef<Path>) -> PathBuf {
		session.as_ref().join(self.name())
	}
}
impl SpecialFile {
	pub fn path(self) -> Result<PathBuf> {
		let root = ws_path()?;
		Ok(match self {
			Self::AiTemplateDir => root.join("ai").join("template"),
			Self::LogDir => root.join("log"),
			Self::TmpDir => root.join("log").join("tmp"),
			Self::SdlcCurrent => root.join("log").join("tmp").join("sdlc.current.json"),
			Self::SessionsDir => root.join("log").join("session"),
			Self::SessionsIndex => root.join("log").join("sessions.json"),
		})
	}
}
impl SpecialFiles {
	pub fn ensure_dir(path: impl Into<PathBuf>) -> Result<PathBuf> {
		let path = path.into();

		fs::create_dir_all(&path)?;

		Ok(path)
	}

	pub fn ensure_parent(path: impl Into<PathBuf>) -> Result<PathBuf> {
		let path = path.into();

		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}

		Ok(path)
	}

	pub fn read(path: impl AsRef<Path>) -> Result<String> {
		Ok(fs::read_to_string(path)?)
	}

	pub fn write(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = Self::ensure_parent(path)?;

		fs::write(path, contents)?;

		Ok(())
	}

	pub fn append(path: impl Into<PathBuf>, contents: impl AsRef<[u8]>) -> Result<()> {
		let path = Self::ensure_parent(path)?;

		let mut file = fs::OpenOptions::new()
			.create(true)
			.append(true)
			.open(path)?;

		file.write_all(contents.as_ref())?;

		Ok(())
	}

	pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
		let contents = Self::read(path)?;

		Ok(serde_json::from_str(&contents)?)
	}

	pub fn write_json<T: Serialize>(path: impl Into<PathBuf>, value: &T) -> Result<()> {
		let contents = serde_json::to_string_pretty(value)?;

		Self::write(path, contents)
	}

	pub fn remove_if_exists(path: impl AsRef<Path>) -> Result<()> {
		let path = path.as_ref();

		if path.exists() {
			fs::remove_file(path)?;
		}

		Ok(())
	}
	pub fn read_session(session: impl AsRef<Path>, file: SessionFile) -> Result<String> {
		Self::read(file.path(session))
	}

	pub fn write_session(
		session: impl AsRef<Path>,
		file: SessionFile,
		contents: impl AsRef<[u8]>,
	) -> Result<()> {
		Self::write(file.path(session), contents)
	}

	pub fn append_session(
		session: impl AsRef<Path>,
		file: SessionFile,
		contents: impl AsRef<[u8]>,
	) -> Result<()> {
		Self::append(file.path(session), contents)
	}
}

pub struct SpecialFiles;
