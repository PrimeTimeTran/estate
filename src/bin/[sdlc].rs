use estate::modules::sdlc::*;
use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenvy::dotenv()?;
	let client = TypeSafeClient::from_env()?;
	let mut sdlc = Sdlc::load(client)?.expect("SDLC state should always exist");
	if sdlc.stage().is_none() {
		sdlc.start(prompt_for_intent()?).await?;
	}
	sdlc.run().await?;
	Ok(())
}
