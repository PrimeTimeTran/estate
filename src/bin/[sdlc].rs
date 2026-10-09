use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::io::IsTerminal;

use estate::prelude::*;

// 1. Normal run
// cargo -q run --bin sdlc --features sdlc
//
// 2. Dry run (no ai invocation)
// DRY_RUN=1 cargo -q run --bin sdlc --features sdlc
//
// let demo = std::env::var_os("DRY_RUN").is_some();
//
// 3. Disable TUI
// SDLC_PLAIN=1 cargo run --bin sdlc --features sdlc
// cargo -q run --bin sdlc --features sdlc
//
// 4. Bypass decide
// rm -rf log && \
// SDLC_RESUME_BUILD=1 SDLC_FORCE_CONTINUE=1 SDLC_PLAIN=1 \
// cargo run --bin sdlc --features sdlc
// SDLC_PLAIN=1 cargo run --bin sdlc --features sdlc
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;

	let ctx = Arc::new(CtxSdlc::default());
	let handle = tokio::runtime::Handle::current();
	let events = EventBus::new();
	let native_runtime = NativeRuntime::new(ctx, handle, events.clone())?;
	native_runtime.start_dispatcher();
	let pipeline = Pipeline::new("Do the work required to build this CLI", events.clone())
		.await
		.context("Pipeline::new")?;

	let mut runtime = PipelineRuntime::<CtxSdlc>::new(pipeline, native_runtime);
	println!(">>> pipeline created");
	println!(">>> creating runtime");
	println!(">>> runtime created");

	let mut events = runtime.pipeline.subscribe();
	let mut view = AiView::new(&runtime);

	let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel::<SdlcInput>();

	let use_tui = std::env::var_os("SDLC_TUI").is_some();

	// if !use_tui {
	// println!(">>> starting runtime");
	// runtime.run(&mut input_rx).await.context("runtime.run")?;
	//
	// println!(">>> runtime finished");
	// return Ok(());
	// }
	let resume_from_build = std::env::var_os("SDLC_RESUME_BUILD").is_some();

	use std::time::{Duration, Instant};
	let session_started_at = chrono::Local::now().format("%H:%M:%S").to_string();
	let session_started = Instant::now();

	if !use_tui {
		println!(">>> starting runtime");

		let mut stdout = std::io::stdout();
		let input = String::new();

		render_plain_guard(&input, &mut stdout, &session_started_at, session_started)?;

		// Refresh the bottom-right ticker once per second.
		let ticker = tokio::spawn(async move {
			let mut stdout = std::io::stdout();
			let mut interval = tokio::time::interval(Duration::from_secs(1));

			loop {
				interval.tick().await;

				if render_plain_guard("", &mut stdout, &session_started_at, session_started).is_err() {
					break;
				}
			}
		});

		let result = if resume_from_build {
			println!(">>> resuming from Build");
			runtime
				.resume_from(Stage::Build, &mut input_rx)
				.await
				.context("runtime.resume_from(Build)")
		} else {
			runtime.run(&mut input_rx).await.context("runtime.run")
		};

		ticker.abort();

		println!(">>> runtime finished");
		result?;
		return Ok(());
	}
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
		AiView::render(frame, &view);
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
												AiView::render(frame, &view);
										})?;
								}

								Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
										view.apply(SdlcEvent::Failed {
												stage: Some(view.runtime.stage),
												error: format!("TUI event receiver lagged by {n} events"),
										});

										terminal.draw(|frame| {
												AiView::render(frame, &view);
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
								AiView::render(frame, &view);
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
fn render_plain_guard(
	input: &str,
	stdout: &mut std::io::Stdout,
	session_started_at: &str,
	session_started: Instant,
) -> std::io::Result<()> {
	use crossterm::{
		QueueableCommand, cursor,
		style::{Color, Print, ResetColor, SetForegroundColor},
		terminal::{self, ClearType},
	};

	let (width, height) = terminal::size()?;
	let row = height.saturating_sub(1);

	let elapsed = session_started.elapsed().as_secs();
	let elapsed_time = format!(
		"{:02}:{:02}:{:02}",
		elapsed / 3600,
		(elapsed % 3600) / 60,
		elapsed % 60,
	);

	let ticker = format!("{session_started_at}  {elapsed_time}");
	let ticker_width = ticker.len();

	stdout.queue(cursor::MoveTo(0, row))?;
	stdout.queue(terminal::Clear(ClearType::CurrentLine))?;

	stdout.queue(SetForegroundColor(Color::Green))?;
	stdout.queue(Print("> "))?;
	stdout.queue(ResetColor)?;

	let input_width = (width as usize).saturating_sub(ticker_width + 3);

	let visible_input: String = input.chars().take(input_width).collect();
	stdout.queue(Print(visible_input))?;

	if width as usize > ticker_width + 2 {
		stdout.queue(cursor::MoveTo(
			width.saturating_sub(ticker_width as u16),
			row,
		))?;
		stdout.queue(SetForegroundColor(Color::Cyan))?;
		stdout.queue(Print(ticker))?;
		stdout.queue(ResetColor)?;
	}

	stdout.queue(cursor::Show)?;
	stdout.flush()
}
