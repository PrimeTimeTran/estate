use anyhow::Context as CtxAnyhow;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use estate::prelude::{sdlc::Context as CtxSdlc, *};
use std::io::IsTerminal;

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
// SDLC_RESUME=1 SDLC_FORCE_CONTINUE=1 SDLC_PLAIN=1 \
// cargo run --bin sdlc --features sdlc
// SDLC_PLAIN=1 cargo run --bin sdlc --features sdlc

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenvy::dotenv().ok();
	let cli = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;

	// 1. Initialize shared execution infrastructure.
	let ctx = Arc::new(CtxSdlc::default());
	let handle = tokio::runtime::Handle::current();
	let event_bus = EventBus::new();

	let host_runtime = NativeRuntime::new(ctx, handle, event_bus.clone())?;
	host_runtime.start_dispatcher();

	// 2. Initialize runtime, session, and pipeline.
	let resume = std::env::var_os("SDLC_RESUME").is_some();

	let (mut runtime, mut pipeline) =
		_Runtime::<CtxSdlc>::initialize(SDLC_GOAL, resume, host_runtime, event_bus)
			.await
			.context("_Runtime::initialize")?;

	// 3. Prepare observers and input.
	let mut events = runtime.event_bus.subscribe();
	let mut view = AiView::new(&runtime, &pipeline);

	let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel::<SdlcInput>();

	let use_tui = std::env::var_os("SDLC_TUI").is_some();
	let is_real_run = std::env::var_os("DRY_RUN").is_none();

	let strategy = if resume {
		// RunStrategy::ResumeFrom(Stage::Plan)
		RunStrategy::ResumeFrom(Stage::Build)
	} else {
		RunStrategy::Full
	};

	let session_started_at = chrono::Local::now().format("%H:%M:%S").to_string();
	let session_started = Instant::now();

	// 4. Plain mode: run and exit.
	if !use_tui || !std::io::stdout().is_terminal() {
		let mut stdout = std::io::stdout();
		let input = String::new();
		render_plain_guard(&input, &mut stdout, &session_started_at, session_started)?;
		let result = run_with_strategy(&mut runtime, &mut pipeline, &mut input_rx, strategy)
			.await
			.context("run_with_strategy");

		println!(">>> runtime finished");

		let _outcome = result?;
		return Ok(());
	}

	// 5. TUI mode: create the runner once.
	let mut runner = _Runner {
		runtime: &mut runtime,
		pipeline: &mut pipeline,
	};

	let run = async {
		if is_real_run {
			run_with_strategy(runner.runtime, runner.pipeline, &mut input_rx, strategy)
				.await
				.map(|_| ())
				.context("run_with_strategy")
		} else {
			runner
				.run_simulated(&mut input_rx)
				.await
				.map(|_| ())
				.context("runner.run_simulated")
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

pub async fn run_with_strategy<C: Ctx>(
	runtime: &mut _Runtime<C>,
	pipeline: &mut Pipeline,
	input_rx: &mut UnboundedReceiver<SdlcInput>,
	strategy: RunStrategy,
) -> Result<RunResults> {
	let mut runner = _Runner { runtime, pipeline };

	match strategy {
		RunStrategy::Full => {
			let outcome = runner.run(input_rx).await.context("_Runner::run")?;

			Ok(RunResults::Pipeline(outcome))
		}
		RunStrategy::ResumeFrom(stage) => {
			let outcome = runner
				.resume_from(stage, input_rx)
				.await
				.with_context(|| format!("_Runner::resume_from({stage:?})"))?;

			Ok(RunResults::Pipeline(outcome))
		}
		RunStrategy::Stage(stage) => {
			let outcome = runner
				.run_stage(stage)
				.await
				.with_context(|| format!("_Runner::run_stage({stage:?})"))?;

			Ok(RunResults::Stage(outcome))
		}
		RunStrategy::SingleAttempt(stage) => {
			let outcome = runner
				.execute_attempt(stage)
				.await
				.with_context(|| format!("_Runner::execute_attempt({stage:?})"))?;

			Ok(RunResults::Attempt(outcome))
		}
	}
}
