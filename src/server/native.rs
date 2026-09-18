use crate::prelude::*;

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
