use crate::{native, prelude::*};

#[derive(Clone, Debug)]
pub struct NativeStateStore;
impl NativeStateStore {
	pub fn new() -> Result<Self> {
		Ok(Self {})
	}
}

impl StateStore for NativeStateStore {
	fn load(&self) -> Result<EstateState> {
		let path = resolver::engine_data_dir()?.join("state.json");

		if !path.exists() {
			tracing::warn!("EstateState does not exist: {:?}", path);
			return Ok(EstateState::default());
		}

		let raw = fs::read_to_string(&path)?;

		if raw.trim().is_empty() {
			tracing::warn!("EstateState is empty: {:?}", path);
			return Ok(EstateState::default());
		}

		Ok(serde_json::from_str(&raw)?)
	}
	fn save(&self, state: &EstateState) -> Result<()> {
		let path = native::resolver::engine_data_dir()?.join("state.json");

		let json = serde_json::to_string_pretty(state)?;
		fs::write(path, json)?;

		Ok(())
	}
}

impl EstateState {
	pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
		let contents = fs::read_to_string(path)?;
		Ok(serde_json::from_str(&contents)?)
	}
	pub fn path() -> std::io::Result<PathBuf> {
		Ok(crate::native::resolver::engine_data_dir()?.join("state.json"))
	}
	pub fn load_from_disk() -> Result<Self> {
		let path = Self::path()?;

		if !path.exists() {
			tracing::warn!("EstateState does not exist: {:?}", path);
			return Ok(Self::default());
		}

		let raw = fs::read_to_string(&path)?;
		Ok(serde_json::from_str(&raw)?)
	}

	pub fn save_to_disk(&self) -> Result<()> {
		let path = Self::path()?;
		tracing::debug!("💾 EstateState saving: {:?}", path);
		let json = serde_json::to_string_pretty(self)?;
		fs::write(path, json)?;
		Ok(())
	}
}
