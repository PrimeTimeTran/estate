use crate::prelude::*;

pub mod cargo;
pub mod logger;

pub use cargo::*;

pub fn read_from_session(name: &str, session: &AiSession) -> anyhow::Result<String> {
	read_from_disk(session.dir.join(name))
}
pub fn read_from_disk(path: impl AsRef<Path>) -> anyhow::Result<String> {
	std::fs::read_to_string(path).map_err(Into::into)
}
