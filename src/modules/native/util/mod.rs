use crate::prelude::*;

pub mod cargo;
pub mod logger;
pub use cargo::*;

use crate::sdlc::SdlcSession;

pub fn read_from_disk(path: impl AsRef<Path>) -> anyhow::Result<String> {
	std::fs::read_to_string(path).map_err(Into::into)
}

pub fn read_from_session(name: &str, session: &SdlcSession) -> anyhow::Result<String> {
	read_from_disk(session.dir.join(name))
}

// pub fn read_(name: &str, session: &SdlcSession) -> anyhow::Result<String> {
// 	read_from_disk(session.dir.join(name))
// }
