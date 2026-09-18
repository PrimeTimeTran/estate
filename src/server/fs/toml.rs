use crate::prelude::*;

impl<T> TomlFile<T>
where
	T: DeserializeOwned,
{
	pub fn new(path: impl Into<PathBuf>) -> Self {
		Self {
			path: path.into(),
			_marker: PhantomData,
		}
	}

	pub fn path(&self) -> &Path {
		&self.path
	}

	pub async fn read(&self) -> Result<T> {
		let toml = tokio::fs::read_to_string(&self.path).await?;
		Ok(toml::from_str(&toml)?)
	}
}

impl<T> TomlFile<T>
where
	T: Serialize,
{
	pub async fn write(&self, value: &T) -> Result<()> {
		let toml = toml::to_string_pretty(value)?;
		tokio::fs::write(&self.path, toml).await?;
		Ok(())
	}
}

impl<T> TomlFile<T>
where
	T: DeserializeOwned,
{
	pub fn read_sync(&self) -> Result<T> {
		let toml = std::fs::read_to_string(&self.path)?;
		Ok(toml::from_str(&toml)?)
	}
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TomlFile<T> {
	path: PathBuf,
	_marker: PhantomData<T>,
}
