use crate::{prelude::*, proto::types::*};
use tonic::{Request, Response, Status};

pub mod events;
pub mod json;
pub mod native;
pub mod problem;
pub mod submission;

pub use crate::server::{events::*, json::*, native::*, problem::*, submission::*};

pub fn internal_error(error: anyhow::Error) -> Status {
	tracing::error!("{error:#}");
	Status::internal(error.to_string())
}
pub fn page_request(request: Option<PageRequest>) -> Result<PageRequest, Status> {
	request.ok_or_else(|| Status::invalid_argument("page is required"))
}

impl<T> JsonFile<T>
where
	T: Serialize + DeserializeOwned,
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
		let json = tokio::fs::read_to_string(&self.path).await?;
		Ok(serde_json::from_str(&json)?)
	}

	pub async fn write(&self, value: &T) -> Result<()> {
		let json = serde_json::to_string_pretty(value)?;
		tokio::fs::write(&self.path, json).await?;
		Ok(())
	}

	pub async fn update<F>(&self, update: F) -> Result<T>
	where
		F: FnOnce(&mut T),
	{
		let mut value = self.read().await?;
		update(&mut value);
		self.write(&value).await?;
		Ok(value)
	}

	pub async fn delete(&self) -> Result<()> {
		tokio::fs::remove_file(&self.path).await?;
		Ok(())
	}
}

pub struct JsonFile<T> {
	path: PathBuf,
	_marker: PhantomData<T>,
}
