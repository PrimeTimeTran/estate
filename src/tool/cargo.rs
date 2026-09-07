use crate::prelude::*;
use std::{
	path::{Path, PathBuf},
	sync::mpsc,
};

// use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

fn read_cargo_toml() -> CargoToml {
	let raw = fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
	toml::from_str(&raw).expect("invalid Cargo.toml")
}

fn write_cargo_toml(cfg: &CargoToml) {
	let toml_str = toml::to_string(cfg).unwrap();
	fs::write("Cargo.toml", toml_str).expect("failed to write Cargo.toml");
}

pub fn sync_project_metadata() {
	let mut cargo = read_cargo_toml();
	cargo.package.version = "0.1.1".to_string();
	write_cargo_toml(&cargo);
}

pub fn derive_runtime_context() {
	let cargo = read_cargo_toml();
	println!("leviticus running in project:");
	println!("  name: {}", cargo.package.name);
	println!("  version: {}", cargo.package.version);
	println!("  edition: {}", cargo.package.edition);
}

fn find_cargo_toml() -> Option<PathBuf> {
	let mut dir = std::env::current_dir().ok()?;

	loop {
		let cargo = dir.join("Cargo.toml");

		if cargo.is_file() {
			return Some(cargo);
		}

		if !dir.pop() {
			return None;
		}
	}
}

impl CargoWatcher {
	pub fn new() -> Result<Self> {
		let path = find_cargo_toml().ok_or_else(|| anyhow::anyhow!("could not find Cargo.toml"))?;

		Ok(Self {
			cargo: TomlFile::new(path),
		})
	}

	pub fn path(&self) -> &Path {
		self.cargo.path()
	}

	pub async fn read(&self) -> Result<CargoManifest> {
		self.cargo.read().await
	}

	pub async fn run_once(&self) -> Result<()> {
		let cargo = self.read().await?;

		tracing::info!("Cargo.toml default feature: {:?}", cargo.default_feature());

		Ok(())
	}

	pub fn read_sync(&self) -> Result<CargoManifest> {
		self.cargo.read_sync()
	}

	pub fn run_once_sync(&self) -> Result<()> {
		let cargo = self.read_sync()?;

		tracing::info!("Cargo.toml default feature: {:?}", cargo.default_feature());

		Ok(())
	}
}
impl CargoManifest {
	pub fn default_feature(&self) -> Option<&str> {
		self
			.features
			.as_ref()?
			.default
			.as_ref()?
			.first()
			.map(String::as_str)
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct CargoFeatures {
	pub default: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct CargoWatcher {
	cargo: TomlFile<CargoManifest>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CargoManifest {
	pub package: Package,
	pub features: Option<CargoFeatures>,
}

#[derive(Deserialize, Serialize)]
struct CargoToml {
	package: Package,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Package {
	pub name: String,
	pub version: String,
	pub edition: String,
}
