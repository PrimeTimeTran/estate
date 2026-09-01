use crate::{model::*, prelude::*};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
	#[default]
	Rust,
	Python,
	JavaScript,
}

impl Language {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Rust => "rust",
			Self::Python => "python",
			Self::JavaScript => "javascript",
		}
	}

	pub fn entry(self) -> &'static str {
		match self {
			Self::Rust => "solution.rs",
			Self::Python => "solution.py",
			Self::JavaScript => "solution.js",
		}
	}

	pub fn from_arg(arg: Option<&str>) -> anyhow::Result<Self> {
		match arg {
			Some("rust") => Ok(Self::Rust),
			Some("python") | Some("py") => Ok(Self::Python),
			Some("javascript") | Some("js") => Ok(Self::JavaScript),
			Some(lang) => {
				anyhow::bail!("unknown language '{lang}', expected rust, python, or javascript")
			}
			None => Ok(Self::Rust),
		}
	}
}

impl From<Language> for ProtoLanguage {
	fn from(value: Language) -> Self {
		match value {
			Language::Rust => Self::Rust,
			Language::Python => Self::Python,
			Language::JavaScript => Self::Javascript,
		}
	}
}

impl TryFrom<ProtoLanguage> for Language {
	type Error = anyhow::Error;

	fn try_from(value: ProtoLanguage) -> Result<Self, Self::Error> {
		match value {
			ProtoLanguage::Rust => Ok(Self::Rust),
			ProtoLanguage::Python => Ok(Self::Python),
			ProtoLanguage::Javascript => Ok(Self::JavaScript),
			ProtoLanguage::Typescript => {
				anyhow::bail!("typescript is not currently supported")
			}
		}
	}
}

impl TryFrom<i32> for Language {
	type Error = anyhow::Error;

	fn try_from(value: i32) -> Result<Self, Self::Error> {
		let proto =
			ProtoLanguage::try_from(value).map_err(|_| anyhow::anyhow!("invalid language: {value}"))?;

		Self::try_from(proto)
	}
}

impl Language {
	pub fn as_proto_i32(self) -> i32 {
		ProtoLanguage::from(self) as i32
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
	Easy,
	Medium,
	Hard,
}

impl TryFrom<i32> for Difficulty {
	type Error = anyhow::Error;

	fn try_from(value: i32) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(Self::Easy),
			2 => Ok(Self::Medium),
			3 => Ok(Self::Hard),
			0 => anyhow::bail!("difficulty unspecified"),
			other => anyhow::bail!("unknown difficulty: {other}"),
		}
	}
}
