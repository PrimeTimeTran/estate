use crossterm::{
	event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
	execute,
	terminal::{EnterAlternateScreen, enable_raw_mode},
};
use estate::prelude::*;

use anyhow::{Context, anyhow};

// 1. Normal run
// cargo -q run --bin sdlc --features sdlc
//
// 2. Dry run (no ai invocation)
// ESTATE_SDLC_DEMO=1 cargo -q run --bin sdlc --features sdlc
//
// let demo = std::env::var_os("ESTATE_SDLC_DEMO").is_some();
//
// cargo -q run --bin sdlc --features sdlc
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut sdlc = Sdlc::init()
		.context("loading SDLC")?
		.ok_or_else(|| anyhow!("no SDLC instance"))?;
	if sdlc.stage().is_none() {
		sdlc.start(prompt_for_intent()?).await?;
	}
	let mut events = sdlc.subscribe();
	let mut view = SdlcView::new(sdlc.stage().unwrap_or(Stage::Intent));

	enable_raw_mode()?;
	let mut stdout = stdout();
	execute!(stdout, EnterAlternateScreen)?;

	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	terminal.clear()?;
	terminal.hide_cursor()?;

	let _terminal_guard = TerminalGuard;

	let (input_tx, input_rx) = tokio::sync::mpsc::unbounded_channel::<SdlcInput>();
	let demo = std::env::var_os("ESTATE_SDLC_DEMO").is_some();
	let run = async {
		if demo {
			sdlc.run_simulated(input_rx).await
		} else {
			sdlc.run(input_rx).await
		}
	};
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
						if key.code == KeyCode::Char('c')
							&& key.modifiers.contains(KeyModifiers::CONTROL)
						{
							break Ok(());
						}
						if view.is_input_active() {
							view.handle_input_key(key, &input_tx)?;
						} else {
							match key.code {
								KeyCode::Char('p') => {
									view.toggle_pause();
								}
								KeyCode::Char('l') => {
									view.toggle_logs();
								}
								KeyCode::Char('r') => {
									let _ = input_tx.send(SdlcInput::Retry);
								}
								KeyCode::Char('v') => {
									let _ = input_tx.send(SdlcInput::Reviewed);
								}
								_ => {}
							}
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
	result?;
	Ok(())
}
