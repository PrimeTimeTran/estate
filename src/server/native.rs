use crate::app::{state::EstateState, *};

use crate::{app::*, prelude::*, services::*};

impl StateService {
	pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
		Self {
			repo: JsonRepo::new(path),
		}
	}
	pub async fn load(&self) -> Result<EstateState> {
		self.repo.read().await
	}
	pub async fn save(&self, state: &EstateState) -> Result<()> {
		self.repo.write(state).await
	}
	pub async fn update<F>(&self, update: F) -> Result<EstateState>
	where
		F: FnOnce(&mut EstateState),
	{
		self.repo.update(update).await
	}
}

impl SessionService {
	pub fn new(state_service: Arc<StateService>) -> Self {
		Self { state_service }
	}
	pub async fn create(&self) -> Result<Session> {
		let session = Session::default();

		self
			.state_service
			.update(|state| {
				state.session = session.clone();
			})
			.await?;

		Ok(session)
	}
	pub async fn end(&self) -> Result<()> {
		tracing::info!("SessionService end");
		self
			.state_service
			.update(|state| {
				state.session.end();
			})
			.await?;

		Ok(())
	}
}

impl<T> JsonRepo<T>
where
	T: Serialize + DeserializeOwned,
{
	pub fn new(path: impl Into<PathBuf>) -> Self {
		Self {
			path: path.into(),
			_marker: std::marker::PhantomData,
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
