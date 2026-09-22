use crossterm::{
	event::{self, Event, KeyCode, KeyEventKind},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use estate::prelude::*;
use jev_sdk::TypeSafeClient;

use anyhow::{Context, anyhow};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenvy::dotenv().ok();

	// 1. Normal run
	// cargo run --bin sdlc
	//
	// 2. Dry run (no ai invocation)
	// ESTATE_SDLC_DEMO=1 cargo run --bin sdlc
	//
	let demo = std::env::var_os("ESTATE_SDLC_DEMO").is_some();

	let client = TypeSafeClient::from_env().context("creating TypeSafe client")?;

	let mut sdlc = Sdlc::load(client)
		.context("loading SDLC")?
		.ok_or_else(|| anyhow!("no SDLC instance"))?;

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
	terminal.clear()?;
	terminal.hide_cursor()?;

	// ------------------------------------------------------------
	// SDLC task
	// ------------------------------------------------------------
	let (input_tx, input_rx) = tokio::sync::mpsc::unbounded_channel::<SdlcInput>();
	let run = async {
		// if demo {
		// 	sdlc.run_simulated(input_rx).await
		// } else {
		sdlc.run(input_rx).await
		// }
	};
	// let run = sdlc.run(input_rx).await?;
	// sdlc.run(input_rx).await

	tokio::pin!(run);

	let mut ticker = tokio::time::interval(Duration::from_millis(100));

	let result = loop {
		tokio::select! {
			result = &mut run => {
				break result;
			}

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

			_ = ticker.tick() => {
				if event::poll(Duration::from_millis(0))? {
					if let Event::Key(key) = event::read()? {
						if key.kind == KeyEventKind::Press {
							match key.code {
								KeyCode::Char('q') => {
									break Ok(());
								}

								KeyCode::Char('p') => {
									view.toggle_pause();
								}

								KeyCode::Char('l') => {
									view.toggle_logs();
								}

								KeyCode::Char('r') => {
									let _ = input_tx.send(
										SdlcInput::Retry
									);
								}

								KeyCode::Char('v') => {
									let _ = input_tx.send(
										SdlcInput::Reviewed
									);
								}

								_ => {}
							}
						}
					}
				}

				terminal.draw(|frame| {
					SdlcView::render(frame, &view);
				})?;
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
