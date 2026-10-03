use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use estate::prelude::*;
use std::io::IsTerminal;
// 1. Normal run
// cargo -q run --bin sdlc --features sdlc
//
// 2. Dry run (no ai invocation)
// DRY_RUN=1 cargo -q run --bin sdlc --features sdlc
//
// let demo = std::env::var_os("DRY_RUN").is_some();
//
// cargo -q run --bin sdlc --features sdlc
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut pipeline = SprintPipeline::new().await.context("loading SDLC")?;
	if pipeline.stage().is_none() {
		pipeline
			.init("Do the work required to build this CLI")
			.await?;
	}
	let mut runtime = PipelineRuntime::new(pipeline);
	let mut events = runtime.pipeline.subscribe();
	let mut view = SdlcView::new(&runtime);
	let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel::<SdlcInput>();
	let is_real_run = std::env::var_os("DRY_RUN").is_none();
	let run = async {
		if is_real_run {
			runtime.run(&mut input_rx).await
		} else {
			runtime.run_simulated(&mut input_rx).await
		}
	};
	tokio::pin!(run);
	if !std::io::stdout().is_terminal() {
		return Ok(());
	}
	let mut terminal =
		ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
	crossterm::terminal::enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	stdout.flush()?;
	crossterm::terminal::disable_raw_mode()?;
	crossterm::terminal::enable_raw_mode()?;
	terminal.clear()?;

	let _guard = Guard;

	terminal.draw(|frame| {
		SdlcView::render(frame, &view);
	})?;

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

										terminal.draw(|frame| {
												SdlcView::render(frame, &view);
										})?;
								}

								Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
										view.apply(SdlcEvent::Failed {
												stage: Some(view.runtime.stage),
												error: format!("TUI event receiver lagged by {n} events"),
										});

										terminal.draw(|frame| {
												SdlcView::render(frame, &view);
										})?;
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
																KeyCode::Char('p') => view.toggle_pause(),
																KeyCode::Char('l') => view.toggle_logs(),

																KeyCode::Char('r') => {
																		input_tx.send(SdlcInput::Retry)?;
																}

																KeyCode::Char('v') => {
																		input_tx.send(SdlcInput::Reviewed)?;
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

	drop(_guard);

	println!();
	println!("✓ SDLC complete");
	println!();
	println!("Intent → Spec → Plan → Build → Verify → Complete");
	println!();
	Ok(())
}

struct Guard;
impl Drop for Guard {
	fn drop(&mut self) {
		let _ = crossterm::terminal::disable_raw_mode();
		let mut stdout = std::io::stdout();

		let _ = crossterm::execute!(
			stdout,
			crossterm::cursor::Show,
			crossterm::terminal::LeaveAlternateScreen
		);
	}
}
