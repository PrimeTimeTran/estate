use anyhow::Context as CtxAnyhow;
use estate::prelude::{sdlc::Context as CtxSdlc, *};
use std::time::Instant;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::io::IsTerminal;
use std::time::Duration;

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
	let event_bus = EventBus::new();

	let native_runtime = NativeRuntime::new(ctx, handle, event_bus.clone())?;
	native_runtime.start_dispatcher();

	let resume_from_build = std::env::var_os("SDLC_RESUME_BUILD").is_some();

	let pipeline = if resume_from_build {
		Pipeline::resume_saved(event_bus.clone())
			.await
			.context("Pipeline::resume_saved")?
	} else {
		Pipeline::new(SDLC_GOAL, event_bus.clone())
			.await
			.context("Pipeline::new")?
	};

	let mut runtime = PipelineRuntime::<CtxSdlc>::new(pipeline, native_runtime);

	let mut events = runtime.event_bus.subscribe();
	let mut view = AiView::new(&runtime);

	let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel::<SdlcInput>();

	let use_tui = std::env::var_os("SDLC_TUI").is_some();
	let is_real_run = std::env::var_os("DRY_RUN").is_none();

	let strategy = if resume_from_build {
		RunStrategy::ResumeFrom(Stage::Plan)
	} else {
		RunStrategy::Full
	};

	let session_started_at = chrono::Local::now().format("%H:%M:%S").to_string();
	let session_started = Instant::now();

	// Plain mode, or TUI requested without an interactive terminal.
	if !use_tui || !std::io::stdout().is_terminal() {
		let mut stdout = std::io::stdout();
		let input = String::new();

		render_plain_guard(&input, &mut stdout, &session_started_at, session_started)?;

		let result = run_with_strategy(&mut runtime, &mut input_rx, strategy)
			.await
			.context("run_with_strategy");

		println!(">>> runtime finished");

		let _outcome = result?;
		return Ok(());
	}

	// The TUI owns rendering and keyboard input, while the same strategy
	// abstraction selects how the pipeline runs.
	let run = async {
		if is_real_run {
			run_with_strategy(&mut runtime, &mut input_rx, strategy)
				.await
				.map(|_| ())
				.context("run_with_strategy")
		} else {
			runtime
				.run_simulated(&mut input_rx)
				.await
				.map(|_| ())
				.context("runtime.run_simulated")
		}
	};

	tokio::pin!(run);

	let mut terminal =
		ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;

	crossterm::terminal::enable_raw_mode()?;

	let mut stdout = std::io::stdout();
	stdout.flush()?;

	// Preserve the existing terminal initialization sequence.
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
						if let e::EventKind::Sdlc(sdlc_event) = event.kind {
							view.apply(sdlc_event);

							terminal.draw(|frame| {
								AiView::render(frame, &view);
							})?;
						}
					}

					Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
						view.apply(SdlcEvent::Failed {
							stage: Some(view.runtime.stage),
							error: format!(
								"TUI event receiver lagged by {n} events"
							),
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
									KeyCode::Char('p') => {
										view.toggle_pause();
									}

									KeyCode::Char('l') => {
										view.toggle_logs();
									}

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

pub async fn run_with_strategy(
	runtime: &mut PipelineRuntime<impl Ctx>,
	input_rx: &mut UnboundedReceiver<SdlcInput>,
	strategy: RunStrategy,
) -> Result<RunResults> {
	match strategy {
		RunStrategy::Full => {
			let outcome = runtime.run(input_rx).await.context("runtime.run")?;
			Ok(RunResults::Pipeline(outcome))
		}

		RunStrategy::ResumeFrom(stage) => {
			let outcome = runtime
				.resume_from(stage, input_rx)
				.await
				.with_context(|| format!("runtime.resume_from({stage:?})"))?;

			Ok(RunResults::Pipeline(outcome))
		}
		RunStrategy::Stage(stage) => {
			let outcome = runtime
				.run_stage(stage)
				.await
				.with_context(|| format!("runtime.run_stage({stage:?})"))?;
			Ok(RunResults::Stage(outcome))
		}

		RunStrategy::SingleAttempt(stage) => {
			let outcome = runtime
				.execute_attempt(stage)
				.await
				.with_context(|| format!("runtime.execute_attempt({stage:?})"))?;

			Ok(RunResults::Attempt(outcome))
		}
	}
}
