use std::time::Duration;

use crossterm::{
	event::{self, Event, KeyCode},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use estate::sdlc::{Sdlc, SdlcEvent, SdlcView, Stage, prompt_for_intent};
use jev_sdk::TypeSafeClient;
use ratatui::{Terminal, backend::CrosstermBackend};

use std::io::{self, stdout};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenvy::dotenv()?;

	let client = TypeSafeClient::from_env()?;
	let mut sdlc = Sdlc::load(client)?.expect("SDLC state should always exist");

	if sdlc.stage().is_none() {
		sdlc.start(prompt_for_intent()?).await?;
	}

	let mut events = sdlc.subscribe();

	let mut view = SdlcView::new(sdlc.stage().unwrap_or(Stage::Intent));

	// ------------------------------------------------------------
	// terminal setup
	// ------------------------------------------------------------

	enable_raw_mode()?;

	let mut stdout = stdout();

	execute!(stdout, EnterAlternateScreen)?;

	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// ------------------------------------------------------------
	// SDLC task
	// ------------------------------------------------------------

	let run = sdlc.run();
	tokio::pin!(run);

	let mut ticker = tokio::time::interval(Duration::from_millis(100));

	let result = loop {
		tokio::select! {
			// ----------------------------------------------------
			// SDLC finished
			// ----------------------------------------------------

			result = &mut run => {
				break result;
			}

			// ----------------------------------------------------
			// SDLC event
			// ----------------------------------------------------

			event = events.recv() => {
				match event {
					Ok(event) => {
						view.apply(event);
					}

					Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
						view.apply(SdlcEvent::Failed {
							stage: Some(view.runtime.stage),
							error: format!(
								"TUI event receiver lagged by {n} events"
							),
						});
					}

					Err(tokio::sync::broadcast::error::RecvError::Closed) => {
						break Ok(());
					}
				}
			}

			// ----------------------------------------------------
			// redraw
			// ----------------------------------------------------

			_ = ticker.tick() => {

				terminal.draw(|frame| {
					SdlcView::render(frame, &view);
				})?;
			}

			// ----------------------------------------------------
			// keyboard
			// ----------------------------------------------------

			_ = tokio::task::yield_now() => {
				if event::poll(Duration::from_millis(0))? {
					if let Event::Key(key) = event::read()? {
						match key.code {
							KeyCode::Char('q') => {
								break Ok(());
							}

							_ => {}
						}
					}
				}
			}
		}
	};

	// ------------------------------------------------------------
	// terminal teardown
	// ------------------------------------------------------------

	disable_raw_mode()?;

	execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

	terminal.show_cursor()?;

	result?;

	Ok(())
}
