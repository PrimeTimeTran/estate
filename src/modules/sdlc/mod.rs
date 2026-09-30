use crate::{
	model::{
		AgentTask,
		agent::{Agent, AgentContext},
		resolver::*,
		task::TaskResult,
	},
	prelude::*,
};
use anyhow::{Context, anyhow};
use crossterm::{
	cursor,
	event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use egui_plot::Corner;
use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};
pub use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
	widgets::Clear,
};
use std::{io::Stdout, process::Command};
use tokio::time::{Duration, sleep};

// cmd+alt+f
// - Search in all files overlay
// cmd+shift+f
// - Search in all files tab
// cmd+f
// - Search in file

// fn from_session(session: &SdlcSession) -> String {
// 	session.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?
// }

// for step in Step::ALL {
//     println!("{step:?}");
// }

fn format_elapsed(duration: Duration) -> String {
	let total_seconds = duration.as_secs();
	let hours = total_seconds / 3600;
	let minutes = (total_seconds % 3600) / 60;
	let seconds = total_seconds % 60;

	if hours > 0 {
		format!("{hours}h {minutes}m {seconds}s")
	} else if minutes > 0 {
		format!("{minutes}m {seconds}s")
	} else {
		format!("{seconds}s")
	}
}
async fn monitor<F, T>(
	stage: Stage,
	attempt: u32,
	run_started: Instant,
	phase: &'static str,
	future: F,
) -> T
where
	F: Future<Output = T>,
{
	let started = Instant::now();
	tokio::pin!(future);
	let mut ticker = tokio::time::interval(STATUS_INTERVAL);
	loop {
		tokio::select! {
			result = &mut future => {
				return result;
			}
			_ = ticker.tick() => {
			}
		}
	}
}
fn slugify(input: &str) -> String {
	let slug = input
		.to_lowercase()
		.chars()
		.map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
		.collect::<String>();

	let slug = slug
		.split('-')
		.filter(|s| !s.is_empty())
		.take(8)
		.collect::<Vec<_>>()
		.join("-");

	if slug.is_empty() {
		return "untitled".into();
	}

	// Windows reserved device names.
	let reserved = [
		"con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
		"com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
	];

	if reserved.contains(&slug.as_str()) {
		format!("session-{slug}")
	} else {
		slug
	}
}

const DEMO_COMPLETE_DELAY: Duration = Duration::from_secs(3);
const DEMO_EVALUATION_TIME: Duration = Duration::from_secs(2);
const DEMO_EXECUTION_TIME: Duration = Duration::from_secs(5);
const DEMO_RETRY_DELAY: Duration = Duration::from_secs(1);
const FMT_HUMAN_READABLE: &'static str = "%B %-d, %Y at %-I:%M:%S %p UTC";
const MAX_STAGE_ATTEMPTS: u32 = 3;
const STATUS_INTERVAL: Duration = Duration::from_secs(30);

mod ui {
	use crate::sdlc::{SdlcEvent, SdlcPhase, SdlcView, Stage, format_elapsed};
	use ratatui::{
		Frame,
		layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
		style::{Color, Modifier, Style},
		text::{Line, Span},
		widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
	};
	pub fn body(chunk: Rect) -> Vec<Rect> {
		RatatuiLayout::default()
			.direction(Direction::Horizontal)
			.constraints([
				Constraint::Percentage(54),
				Constraint::Length(1),
				Constraint::Percentage(45),
			])
			.split(chunk)
			.to_vec()
	}
	pub fn header(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let elapsed = format_elapsed(view.runtime.started_at.elapsed());
		let line = Line::from(vec![
			Span::styled(
				format!("{:?}", view.runtime.stage),
				Style::default().add_modifier(Modifier::BOLD),
			),
			Span::raw(format!(
				" · attempt #{}/3   {}",
				view.runtime.attempt, elapsed
			)),
			Span::raw("    [q] quit  [p] pause  [l] logs"),
		]);

		frame.render_widget(Paragraph::new(line), area);
	}
	pub fn stepper(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let stages = [
			Stage::Intent,
			Stage::Spec,
			Stage::Plan,
			Stage::Build,
			Stage::Verify,
			Stage::Complete,
		];

		let current = view.runtime.stage;

		let current_style = match view.runtime.phase {
			SdlcPhase::Failed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),

			SdlcPhase::Retrying => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::AwaitingHuman => Style::default()
				.fg(Color::Magenta)
				.add_modifier(Modifier::BOLD),

			_ => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),
		};

		let spinner_text = spinner(view.runtime.stage_started_at.elapsed());

		let mut spans = Vec::new();

		for (index, stage) in stages.iter().copied().enumerate() {
			let (symbol, style) = if stage == current {
				(format!("{spinner_text} "), current_style)
			} else if stage.is_before(current) {
				(
					"✓ ".to_string(),
					Style::default()
						.fg(Color::Green)
						.add_modifier(Modifier::BOLD),
				)
			} else {
				(
					"○ ".to_string(),
					Style::default()
						.fg(Color::DarkGray)
						.add_modifier(Modifier::DIM),
				)
			};

			spans.push(Span::styled(symbol, style));

			spans.push(Span::styled(format!("{stage:?}"), style));

			if index + 1 < stages.len() {
				spans.push(Span::styled("  →  ", Style::default().fg(Color::DarkGray)));
			}
		}

		frame.render_widget(Paragraph::new(Line::from(spans)), area);
	}
	pub fn left_stage_panel(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let runtime = &view.runtime;

		let phase_style = phase_style(runtime.phase);

		let score = runtime
			.score
			.map(|value| format!("{value:.2}"))
			.unwrap_or_else(|| "—".into());

		let confidence = runtime
			.confidence
			.map(|value| format!("{value:.2}"))
			.unwrap_or_else(|| "—".into());

		let passed = match runtime.passed {
			Some(true) => Span::styled(
				"yes",
				Style::default()
					.fg(Color::Green)
					.add_modifier(Modifier::BOLD),
			),

			Some(false) => Span::styled(
				"no",
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
			),

			None => Span::styled("—", Style::default().fg(Color::DarkGray)),
		};

		let status = match runtime.phase {
			SdlcPhase::Executing => {
				let frame = spinner(runtime.stage_started_at.elapsed());

				Line::from(vec![
					Span::styled(
						format!("{frame} "),
						Style::default()
							.fg(Color::Yellow)
							.add_modifier(Modifier::BOLD),
					),
					Span::styled(
						format!("{:?}", runtime.stage),
						Style::default()
							.fg(Color::White)
							.add_modifier(Modifier::BOLD),
					),
					Span::styled(
						format!("   attempt {}/3", runtime.attempt),
						Style::default().fg(Color::Gray),
					),
				])
			}
			SdlcPhase::Retrying => Line::from(vec![
				Span::styled(
					"↻ ",
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("   retrying · attempt {}/3", runtime.attempt),
					Style::default().fg(Color::Gray),
				),
			]),
			SdlcPhase::Evaluating => Line::from(vec![
				Span::styled(
					"◆ ",
					Style::default()
						.fg(Color::Cyan)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled("   evaluating", Style::default().fg(Color::Cyan)),
			]),
			SdlcPhase::AwaitingHuman => Line::from(vec![
				Span::styled(
					"⚠ ",
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled("  Input", Style::default().fg(Color::Gray)),
			]),
			SdlcPhase::Failed => Line::from(vec![
				Span::styled(
					"✗ ",
					Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
				),
				Span::styled("   failed", Style::default().fg(Color::Gray)),
			]),
			SdlcPhase::Starting => Line::from(vec![
				Span::styled("· ", Style::default().fg(Color::Yellow)),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
			]),
			SdlcPhase::Completed => Line::from(vec![
				Span::styled("✓ ", Style::default().fg(Color::Green)),
				Span::styled(
					format!("{:?}", runtime.stage),
					Style::default()
						.fg(Color::Green)
						.add_modifier(Modifier::BOLD),
				),
			]),
		};

		let lines = vec![
			status,
			Line::from(""),
			Line::from(vec![
				Span::styled("phase       ", Style::default().fg(Color::DarkGray)),
				Span::styled(format!("{:?}", runtime.phase), phase_style),
			]),
			Line::from(vec![
				Span::styled("stage time  ", Style::default().fg(Color::DarkGray)),
				Span::styled(
					format_elapsed(runtime.stage_started_at.elapsed()),
					Style::default().fg(Color::White),
				),
			]),
			Line::from(vec![
				Span::styled("JEV score   ", Style::default().fg(Color::DarkGray)),
				Span::styled(score, Style::default().fg(Color::Cyan)),
			]),
			Line::from(vec![
				Span::styled("confidence  ", Style::default().fg(Color::DarkGray)),
				Span::styled(confidence, Style::default().fg(Color::Cyan)),
			]),
			Line::from(vec![
				Span::styled("passed      ", Style::default().fg(Color::DarkGray)),
				passed,
			]),
		];
		frame.render_widget(
			Paragraph::new(lines).block(
				Block::default()
					.borders(Borders::ALL)
					.border_type(BorderType::Rounded)
					.border_style(Style::default().fg(Color::Gray))
					.style(Style::default().bg(Color::Black))
					.title(Span::styled(
						" Current Stage ",
						Style::default()
							.fg(Color::White)
							.add_modifier(Modifier::BOLD),
					)),
			),
			area,
		);
	}
	pub fn right_activity_panel(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		let runtime = &view.runtime;

		let spinner = spinner(runtime.stage_started_at.elapsed());

		let stage_style = Style::default()
			.fg(Color::White)
			.add_modifier(Modifier::BOLD);

		let attempt_style = Style::default().fg(Color::DarkGray);
		let phase_style = phase_style(runtime.phase);

		let phase_label = match runtime.phase {
			SdlcPhase::Starting => "Starting",
			SdlcPhase::Executing => "Executing",
			SdlcPhase::Evaluating => "Evaluating",
			SdlcPhase::Retrying => "Retrying",
			SdlcPhase::AwaitingHuman => "Awaiting input",
			SdlcPhase::Completed => "Completed",
			SdlcPhase::Failed => "Failed",
		};

		let mut lines = Vec::new();

		// ------------------------------------------------------------
		// Current activity
		// ------------------------------------------------------------

		lines.push(Line::from(vec![
			Span::styled(
				format!("{spinner} "),
				Style::default()
					.fg(Color::Yellow)
					.add_modifier(Modifier::BOLD),
			),
			Span::styled(format!("{:?}", runtime.stage), stage_style),
			Span::styled(format!(" · #{}", runtime.attempt), attempt_style),
		]));

		lines.push(Line::from(vec![
			Span::raw("  ↳ "),
			Span::styled(phase_label, phase_style),
		]));

		// ------------------------------------------------------------
		// Current reason
		// ------------------------------------------------------------

		if let Some(message) = &runtime.message {
			lines.push(Line::from(""));
			lines.push(Line::from(Span::styled(
				"  Why",
				Style::default()
					.fg(Color::Gray)
					.add_modifier(Modifier::BOLD),
			)));

			for line in message.lines() {
				lines.push(Line::from(vec![
					Span::raw("     "),
					Span::styled(line, Style::default().fg(Color::DarkGray)),
				]));
			}
		}

		// ------------------------------------------------------------
		// Progression / history
		// ------------------------------------------------------------

		if !runtime.activity.is_empty() {
			lines.push(Line::from(""));
			lines.push(Line::from(Span::styled(
				"  Progress",
				Style::default()
					.fg(Color::Gray)
					.add_modifier(Modifier::BOLD),
			)));

			for activity in runtime.activity.iter().rev().take(8) {
				lines.push(Line::from(vec![
					Span::raw("     • "),
					Span::styled(activity.as_str(), Style::default().fg(Color::White)),
				]));
			}
		}

		frame.render_widget(
			Paragraph::new(lines).wrap(Wrap { trim: true }).block(
				Block::default()
					.borders(Borders::ALL)
					.border_type(BorderType::Plain)
					.border_style(Style::default().fg(Color::DarkGray))
					.title(Span::styled(" Activity ", Style::default().fg(Color::Gray))),
			),
			area,
		);
	}
	pub fn footer(frame: &mut Frame<'_>, view: &SdlcView, area: Rect) {
		if view.is_input_active() {
			let input = Paragraph::new(view.input.as_str())
				.block(
					Block::default()
						.borders(Borders::ALL)
						.title("Human input — Enter to send"),
				)
				.wrap(Wrap { trim: false });
			frame.render_widget(input, area);
			let x = area.x + 1 + view.input.chars().count() as u16;
			let y = area.y + 1;
			frame.set_cursor_position(Position::new(x, y));
		} else {
			let footer = Paragraph::new("p pause  l logs  Ctrl+C quit");
			frame.render_widget(footer, area);
		}
	}
	pub fn event_line(event: &SdlcEvent) -> Line<'static> {
		match event {
			SdlcEvent::HumanInput { stage, input } => Line::from(vec![
				Span::styled("↳ ", Style::default().fg(Color::Magenta)),
				Span::styled(
					format!("{stage:?} human input"),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::raw(": "),
				Span::styled(input.clone(), Style::default().fg(Color::Gray)),
			]),
			SdlcEvent::InterventionRequired {
				stage,
				attempt,
				reason,
			} => Line::from(vec![
				Span::styled(
					"⚠ ",
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!("{stage:?} requires human intervention"),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!(" · attempt #{attempt}"),
					Style::default().fg(Color::Gray),
				),
				Span::raw(format!(": {reason}")),
			]),

			SdlcEvent::InterventionResolved { stage, action } => Line::from(vec![
				Span::styled("✓ ", Style::default().fg(Color::Magenta)),
				Span::styled(
					format!("{stage:?} intervention"),
					Style::default()
						.fg(Color::Magenta)
						.add_modifier(Modifier::BOLD),
				),
				Span::raw(format!(" → {action}")),
			]),
			SdlcEvent::RunStarted => Line::from(Span::styled(
				"SDLC started",
				Style::default().fg(Color::Gray),
			)),

			SdlcEvent::StageStarted { stage, attempt } => Line::from(vec![
				Span::styled("● ", Style::default().fg(Color::White)),
				Span::styled(
					format!("{stage:?}"),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
				Span::styled(
					format!(" · attempt #{attempt}"),
					Style::default().fg(Color::Gray),
				),
			]),

			SdlcEvent::PhaseChanged { phase } => Line::from(vec![
				Span::styled("  phase ", Style::default().fg(Color::DarkGray)),
				Span::styled(format!("→ {phase:?}"), phase_style(*phase)),
			]),

			SdlcEvent::ExecutionComplete { stage } => Line::from(vec![
				Span::styled("✓ ", Style::default().fg(Color::Green)),
				Span::styled(
					format!("{stage:?} execution complete"),
					Style::default().fg(Color::Green),
				),
			]),

			SdlcEvent::ExecutionFailed {
				stage,
				attempt,
				error,
			} => Line::from(vec![
				Span::styled("✗ ", Style::default().fg(Color::Red)),
				Span::styled(
					format!("{stage:?} attempt #{attempt}"),
					Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
				),
				Span::styled(format!(": {error}"), Style::default().fg(Color::Gray)),
			]),

			SdlcEvent::EvaluationStarted { stage } => Line::from(vec![
				Span::styled("◆ ", Style::default().fg(Color::Cyan)),
				Span::styled(
					format!("JEV evaluating {stage:?}"),
					Style::default().fg(Color::Cyan),
				),
			]),

			SdlcEvent::Evaluated {
				stage,
				score,
				confidence,
				passed,
			} => {
				let style = if *passed {
					Style::default()
						.fg(Color::Green)
						.add_modifier(Modifier::BOLD)
				} else {
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD)
				};

				Line::from(vec![
					Span::styled("◆ JEV ", Style::default().fg(Color::Cyan)),
					Span::styled(
						format!("{stage:?} · score={score:.2} confidence={confidence:.2}"),
						Style::default().fg(Color::Gray),
					),
					Span::raw(" · "),
					Span::styled(if *passed { "passed" } else { "rejected" }, style),
				])
			}

			SdlcEvent::EvaluationFailed { stage, error } => Line::from(vec![
				Span::styled("✗ JEV ", Style::default().fg(Color::Red)),
				Span::styled(
					format!("{stage:?}: {error}"),
					Style::default().fg(Color::Gray),
				),
			]),

			SdlcEvent::StageRetrying { stage, number } => Line::from(vec![
				Span::styled("↻ ", Style::default().fg(Color::Yellow)),
				Span::styled(
					format!("{stage:?} · retry #{number}"),
					Style::default()
						.fg(Color::Yellow)
						.add_modifier(Modifier::BOLD),
				),
			]),

			SdlcEvent::StageTransitioned { from, to } => Line::from(vec![
				Span::styled("→ ", Style::default().fg(Color::Cyan)),
				Span::styled(format!("{from:?}"), Style::default().fg(Color::Gray)),
				Span::raw(" → "),
				Span::styled(
					format!("{to:?}"),
					Style::default()
						.fg(Color::White)
						.add_modifier(Modifier::BOLD),
				),
			]),

			SdlcEvent::Completed => Line::from(Span::styled(
				"✓ SDLC complete",
				Style::default()
					.fg(Color::Green)
					.add_modifier(Modifier::BOLD),
			)),

			SdlcEvent::Exited { reason } => Line::from(vec![
				Span::styled("→ exited ", Style::default().fg(Color::DarkGray)),
				Span::styled(reason.clone(), Style::default().fg(Color::Gray)),
			]),

			SdlcEvent::Failed { stage, error } => {
				let message = match stage {
					Some(stage) => format!("{stage:?}: {error}"),
					None => error.clone(),
				};

				Line::from(vec![
					Span::styled("✗ ", Style::default().fg(Color::Red)),
					Span::styled(message, Style::default().fg(Color::Red)),
				])
			}
		}
	}
	pub fn stage_style(stage: Stage, current: Stage) -> Style {
		match stage {
			stage if stage == current => Style::default()
				.fg(Color::White)
				.add_modifier(Modifier::BOLD),

			stage if stage.is_before(current) => Style::default()
				.fg(Color::Green)
				.add_modifier(Modifier::BOLD),

			_ => Style::default()
				.fg(Color::DarkGray)
				.add_modifier(Modifier::DIM),
		}
	}
	pub fn phase_style(phase: SdlcPhase) -> Style {
		match phase {
			SdlcPhase::Starting => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Executing => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Evaluating => Style::default()
				.fg(Color::Cyan)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Retrying => Style::default()
				.fg(Color::Yellow)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::AwaitingHuman => Style::default()
				.fg(Color::Magenta)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Completed => Style::default()
				.fg(Color::Green)
				.add_modifier(Modifier::BOLD),

			SdlcPhase::Failed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
		}
	}
	pub fn spinner(elapsed: std::time::Duration) -> &'static str {
		const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
		let index = (elapsed.as_millis() / 100) as usize % FRAMES.len();
		FRAMES[index]
	}
}
pub mod prompt {
	use super::*;
	pub fn initial_prompt() -> Result<String> {
		Ok(String::from(
			"I need to build a CLI tool. I wnat to use NodeJS.
			Create a file named hello-world.js in the repository root from where I ran this command.
			This file will be the CLI entrypoint. The CLI tool should accept an
			argument and write that value to hello-world.md.
			The user should be able to run node hello-world.js \"hi\".

			- Add tests covering both the JavaScript logic and the CLI behavior.",
		))
	}
	const INTENT_PROMPT: &str = include_str!("../../../ai/template/intent.md");
	const USER_REQUEST: &str = include_str!("../../../ai/template/prompt.md");

	pub fn for_intent(user_request: &str) -> String {
		INTENT_PROMPT.replace("{{USER_REQUEST}}", user_request)
	}

	pub fn tests_gen(intent: &str, spec: &str, plan: &str) -> String {
		format!(
			r#"
			You are designing the verification plan for an SDLC task.

			The user's intent is authoritative.

			## Intent

			{intent}

			## Specification

			{spec}

			## Implementation Plan

			{plan}

			## Instructions

			Create `tests.md`.

			Every requirement in the specification must have at least
			one corresponding verification test.

			Tests should distinguish between:

			1. deterministic checks
				- cargo test
				- cargo check
				- cargo clippy
				- cargo fmt
				- application-specific commands

			2. behavioral tests
				- unit tests
				- integration tests
				- end-to-end tests

			3. semantic verification
				- requirements that cannot be established purely through
					deterministic commands and should later be evaluated by JEV

			Each test must be concrete enough that another agent can implement
			or execute it.

			Use this format:

			# Tests

			## Requirement: <requirement>

			- [ ] <test>

			Do not mark any test as complete.

			Return only the contents of `tests.md`.
		"#,
		)
	}
	pub fn plan_gen(intent: &str, spec: &str) -> String {
		format!(
			r#"
				You are creating an implementation plan for an SDLC system.

				The user's intent is authoritative.

				## Intent

				{intent}

				## Specification

				{spec}

				## Instructions

				Create a concrete implementation plan.

				The plan must:
				- identify the implementation work required
				- break the work into ordered steps
				- identify files/components likely to change
				- identify dependencies between steps
				- identify how each requirement will be verified
				- avoid inventing requirements not present in the intent or specification

				Return only the contents of `plan.md`.
			"#,
		)
	}
}
mod enums {
	use super::*;
	#[derive(Debug, Clone)]
	pub enum Intervention {
		Human(String),
		Retry,
		ProvideContext(String),
		Reviewed,
		Abort,
	}
	#[derive(Debug, Clone, Copy)]
	pub enum GenerationProvider {
		Local,
		Api,
	}
	#[derive(Debug, Clone)]
	pub enum SdlcEvent {
		// RunCompleted,
		Completed,
		Failed {
			stage: Option<Stage>,
			error: String,
		},
		InterventionRequired {
			stage: Stage,
			attempt: Attempt,
			reason: String,
		},
		InterventionResolved {
			stage: Stage,
			action: String,
		},
		Evaluated {
			stage: Stage,
			score: f32,
			confidence: f32,
			passed: bool,
		},
		EvaluationStarted {
			stage: Stage,
		},
		EvaluationFailed {
			stage: Stage,
			error: String,
		},
		ExecutionComplete {
			stage: Stage,
		},
		ExecutionFailed {
			stage: Stage,
			attempt: Attempt,
			error: String,
		},
		Exited {
			reason: String,
		},
		PhaseChanged {
			phase: SdlcPhase,
		},
		RunStarted,
		StageRetrying {
			stage: Stage,
			number: u32,
		},
		StageStarted {
			stage: Stage,
			attempt: Attempt,
		},
		StageTransitioned {
			from: Stage,
			to: Stage,
		},
		HumanInput {
			stage: Stage,
			input: String,
		},
	}
	#[derive(Debug)]
	pub enum SdlcInput {
		Abort,
		ProvideContext(String),
		Retry,
		Reviewed,
		Human(String),
	}
	#[derive(Debug, Clone, Copy, PartialEq)]
	pub enum SdlcPhase {
		Starting,
		Executing,
		Evaluating,
		Retrying,
		AwaitingHuman,
		Completed,
		Failed,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Step {
		Init,
		Intent,
		Spec,
		Plan,
		Build,
		Verify,
		Deploy,
		Maintain,
		Complete,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
	pub enum Stage {
		Intent,
		Spec,
		Plan,
		Build,
		Verify,
		Deploy,
		Maintain,
		Complete,
		SprintCompleted,
	}
	impl std::fmt::Display for Stage {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			let name = match self {
				Self::Intent => "Intent",
				Self::Spec => "Spec",
				Self::Plan => "Plan",
				Self::Build => "Build",
				Self::Verify => "Verify",
				Self::Deploy => "Deploy",
				Self::Maintain => "Maintain",
				Self::Complete => "Complete",
				Self::SprintCompleted => "Sprint Completed",
			};

			f.write_str(name)
		}
	}
	#[derive(Debug, Clone, Copy)]
	pub enum StageAction {
		Continue,
		Retry,
		Intervene,
		Abort,
	}
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum StageActor {
		Human,
		Sdlc,
		Evaluator,
		Agent,
	}
	#[derive(Debug)]
	pub enum StageDecision {
		Continue,

		Retry,

		Revise,

		AwaitHuman,

		Fail,

		Complete,
	}
	pub enum StageOutcome {
		FailExecution,
		// Agent couldn't perform the stage.
		FailEvaluation,
		// Evaluator couldn't evaluate the stage.
		FailQuality,
		// Evaluator worked and said "not good enough."
		FailInfrastructure,
		// Something outside the stage broke.
		/// Stage executed and the resulting artifact passed evaluation.
		Complete {
			result: TaskResult,
			evaluation: StageEvaluation,
		},

		/// Stage executed successfully, but the artifact needs revision.
		NeedsRevision {
			result: TaskResult,
			evaluation: StageEvaluation,
		},

		/// The stage itself could not execute successfully.
		ExecutionFailed(anyhow::Error),

		/// The stage executed, but evaluation could not be completed.
		EvaluationFailed(anyhow::Error),
	}
	pub enum StageOutcomeEvaluation {
		Passed(Evaluation),
		FailedQuality(Evaluation),
		FailedRuntime(Error),
	}
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum StageStatus {
		Running,
		Completed,
		Failed,
		NeedsRevision,
		EvaluationFailed,
	}
	pub enum InputMode {
		Normal,
		Human { buffer: String },
		AwaitingHuman { prompt: String },
	}

	pub enum RunControl {
		Continue,
		Exit,
	}

	pub enum RunState {
		Idle,
		Running,
		Paused,
		AwaitingInput,
		Completed,
		Failed,
		Cancelled,
	}
	pub enum RunResult {
		Completed,
		Failed,
		Cancelled,
	}
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub enum StageResult {
		Intent,
		Spec,
		Plan,
		Build,
		Verification(Verification),
		Complete,
		SprintCompleted,
	}
	pub enum StageTransition {
		Next,
		Repeat,
		Goto(Stage),
		Complete,
		Fail,
		AwaitHuman,
	}
}
mod traits {
	use super::*;
	use crate::{
		model::{
			AgentTask,
			agent::{Agent, AgentContext},
			resolver::*,
			task::TaskResult,
		},
		prelude::*,
	};
	use anyhow::anyhow;
	use crossterm::{
		cursor,
		event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
		execute,
		terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
	};
	use egui_plot::Corner;
	use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};

	use std::{io::Stdout, process::Command};
	use tokio::time::{Duration, sleep};

	struct Context {}

	// Steps to complete the pipeline
	pub trait Pipeline {
		type Stage: Stage;
		fn name(&self) -> &'static str;
		fn description(&self) -> &'static str {
			""
		}
		fn stages(&self) -> &[Self::Stage];
	}
	#[async_trait::async_trait]
	pub trait Runner {
		type Context;
		type Output;

		async fn run(&mut self, ctx: &mut Self::Context) -> Result<Self::Output>;

		fn cancel(&mut self);

		fn is_running(&self) -> bool;
	}
	// 	pub trait Runner {
	// 		type Pipeline: Pipeline;
	// 		type Context;
	//
	// 		/// Execute a pipeline to completion.
	// 		fn run(&mut self, pipeline: &Self::Pipeline, ctx: &mut Self::Context) -> Result<RunResult>;
	//
	// 		/// Request graceful cancellation.
	// 		fn cancel(&mut self);
	//
	// 		/// Current execution state.
	// 		fn state(&self) -> RunState;
	// 	}

	trait Stage {
		fn name(&self) -> &'static str;

		fn description(&self) -> &'static str {
			""
		}

		fn prepare(&self, _ctx: &mut Context) -> Result<()> {
			Ok(())
		}

		fn run(&self, ctx: &mut Context) -> Result<StageResult>;

		fn evaluate(&self, _ctx: &Context, _result: &StageResult) -> Result<Option<Evaluation>> {
			Ok(None)
		}

		fn transition(
			&self,
			_ctx: &Context,
			_result: &StageResult,
			_evaluation: Option<&Evaluation>,
		) -> Result<StageTransition> {
			Ok(StageTransition::Next)
		}

		fn cleanup(&self, _ctx: &mut Context) -> Result<()> {
			Ok(())
		}
	}

	// Identity & Persistence
	#[async_trait::async_trait]
	pub trait ArtifactGenerator: Send + Sync {
		async fn generate(&self, prompt: &str) -> Result<String>;
	}
	pub trait TextModel {
		async fn generate(&self, prompt: &str) -> Result<String>;
	}
}

pub struct SprintRunner<'a> {
	pipeline: &'a mut SprintPipeline,
}
pub struct SprintPipeline {
	evaluator: Evaluator,
	generator: Box<dyn ArtifactGenerator>,
	session: Option<SdlcSession>,
	state_path: PathBuf,

	event_tx: broadcast::Sender<SdlcEvent>,

	last_stage: Option<Stage>,
	stage_attempt: u32,
}
#[derive(Debug, Clone)]
pub struct PipelineRuntime {
	pub activity: Vec<String>,
	pub attempt: u32,
	pub stage: enums::Stage,

	pub started_at: Instant,
	pub stage_started_at: Instant,

	pub phase: SdlcPhase,

	pub score: Option<f32>,
	pub confidence: Option<f32>,
	pub passed: Option<bool>,

	pub message: Option<String>,

	pub total_tokens: u64,
	pub total_agent_calls: u32,

	pub history: Vec<SdlcEvent>,
}

#[derive(Debug)]
pub struct StageExecution {
	pub stage: enums::Stage,
	pub attempt: Attempt,
	pub started_at: DateTime<Utc>,
	pub completed_at: DateTime<Utc>,
	pub result: StageResult,
}
#[derive(Debug)]
pub enum ExecutionResult {
	Completed(TaskResult),
	Failed(StageError),
}
#[derive(Debug)]
pub struct StageError;
#[derive(Debug)]
pub enum StageOutcome {
	Complete {
		execution: StageExecution,
		evaluation: StageEvaluation,
	},
	NeedsRevision {
		execution: StageExecution,
		evaluation: StageEvaluation,
	},
	ExecutionFailed {
		stage: enums::Stage,
		attempt: Attempt,
		error: anyhow::Error,
	},
	EvaluationFailed {
		execution: StageExecution,
		error: anyhow::Error,
	},
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
	pub attempt: Attempt,
	pub stage: enums::Stage,
	pub status: StageStatus,

	/// What actually performed the work.
	pub actor: StageActor,

	pub started_at: DateTime<Utc>,
	pub completed_at: Option<DateTime<Utc>>,

	/// Semantic evaluation of the resulting artifact/work.
	pub evaluation: Option<StageEvaluation>,
}

pub struct RetryPolicy {
	max_attempts: u32,
	retry_execution: bool,
	retry_evaluation: bool,
	retry_quality: bool,
	allow_human_intervention: bool,
}

pub struct Evaluator {
	jev: TypeSafeClient,
}
#[derive(Debug, Clone)]
pub struct Evaluation {
	pub score: f64,
	pub confidence: f64,
	pub meets_bar: bool,
	pub feedback: String,
	pub criteria: Vec<CriterionResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
	pub name: String,
	pub passed: bool,
	pub score: f64,
	pub confidence: f64,
	pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEvaluation {
	pub stage: enums::Stage,
	pub actor: StageActor,

	pub started_at: DateTime<Utc>,
	pub completed_at: DateTime<Utc>,

	pub score: f64,
	pub confidence: f64,
	pub passed: bool,

	pub evaluations: Vec<EvaluationResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
	pub score: f64,
	pub confidence: f64,
	pub meets_bar: bool,
	pub feedback: String,
}

pub struct Check {
	pub name: String,
	pub command: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
	pub name: String,
	pub passed: bool,
	pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
	pub passed: bool,
	// pub score: u32,
	pub checks: Vec<CheckResult>,
	pub evaluations: Vec<EvaluationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdlcSession {
	pub id: String,
	pub title: String,
	pub stage: enums::Stage,
	pub stages: Vec<StageRecord>,
	pub dir: PathBuf,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}
impl SdlcSession {
	pub fn new(title: impl Into<String>, dir: PathBuf) -> Self {
		let now = Utc::now();

		Self {
			id: uuid::Uuid::new_v4().to_string(),
			title: title.into(),
			stage: Stage::Intent,
			stages: Vec::new(),
			dir,
			created_at: now,
			updated_at: now,
		}
	}
}
pub struct SdlcView {
	pub runtime: PipelineRuntime,
	pub paused: bool,
	pub show_logs: bool,
	pub input: String,
	pub input_active: bool,
}
pub struct ApiGenerator {
	// whatever API client you decide to use
}
pub struct LocalGenerator {
	agent: Agent,
}
// }
use enums as e;
pub use enums::*;
pub use prompt as agent_prompts;
use prompt::*;
use structs as s;
use traits as t;

use traits::*;

#[async_trait]
impl ArtifactGenerator for LocalGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		let task = AgentTask::new(prompt.to_string());
		let (event_tx, _) = tokio::sync::mpsc::unbounded_channel();

		let result = self.agent.run_agent_loop(task, event_tx).await?;

		println!("=== GENERATOR RESULT ===");
		println!("status: {:?}", result.status);
		println!("chat: {:?}", result.chat);
		println!("summary: {:?}", result.summary);
		println!("artifacts: {:?}", result.artifacts);
		println!("logs: {:?}", result.logs);
		println!("========================");
		Ok(
			result
				.chat
				.or(result.summary)
				.unwrap_or_else(|| "Agent completed".to_string()),
		)
	}
}
#[async_trait]
impl ArtifactGenerator for ApiGenerator {
	async fn generate(&self, prompt: &str) -> Result<String> {
		todo!("apigenerator generate")
	}
}
impl Evaluator {
	async fn evaluate_intent(&self, intent: &str) -> Result<StageEvaluation> {
		let started_at = Utc::now();

		let response = self
			.jev
			.system_one(
				intent,
				[
					(
						"quality",
						Question::from(Score::new(
							"How well does this intent define a concrete software task?",
							[
								"Unusable: the desired outcome is unclear or not actionable",
								"Weak: some intent is present, but major ambiguity remains",
								"Usable: the intended outcome is understandable with some ambiguity",
								"Strong: the desired outcome is concrete and actionable",
								"Excellent: the desired outcome is precise, bounded, and directly actionable",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does this intent provide a sufficiently clear and concrete \
                         desired outcome for an implementation agent to act on \
                         without inventing major requirements?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no intent quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no intent decision"))?;

		let completed_at = Utc::now();

		let passed = meets_bar.noul >= 0.80 && quality.confidence >= 0.70;

		Ok(StageEvaluation {
			stage: Stage::Intent,
			actor: StageActor::Evaluator,
			started_at,
			completed_at,

			score: quality.score,
			confidence: quality.confidence,
			passed,

			evaluations: vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0, // if Noul doesn't provide confidence
					explanation: String::new(),
				},
			],
		})
	}
	async fn evaluate_spec(&self, intent: &str, spec: &str) -> Result<StageEvaluation> {
		let started_at = Utc::now();

		let state = format!(
			"## User Intent\n\n{intent}\n\n\
         ## Specification\n\n{spec}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How faithfully does the specification translate the intent \
                         into concrete, testable requirements?",
							[
								"Unusable: requirements are missing, contradictory, or unrelated",
								"Weak: substantial requirements are missing or invented",
								"Usable: the main intent is represented but some details are weak",
								"Strong: requirements are concrete, relevant, and testable",
								"Excellent: requirements comprehensively and precisely capture the intent \
                             without inventing unnecessary scope",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the specification faithfully represent the user's intent \
                         and provide sufficiently concrete requirements for planning \
                         and verification?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no spec quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no spec decision"))?;

		Ok(StageEvaluation::new(
			Stage::Spec,
			StageActor::Sdlc,
			started_at,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
	async fn evaluate_plan(
		&self,
		intent: &str,
		spec: &str,
		plan: &str,
		tests: &str,
	) -> Result<StageEvaluation> {
		let started_at = Utc::now();
		let state = format!(
			"## User Intent\n\n{intent}\n\n\
			 ## Specification\n\n{spec}\n\n\
			 ## Implementation Plan\n\n{plan}\n\n\
			 ## Test Plan\n\n{tests}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How well does the implementation and test plan cover \
							 the specification?",
							[
								"Unusable: the plan does not provide a viable path to implementation",
								"Weak: major requirements or verification steps are uncovered",
								"Usable: the main implementation and verification work is covered",
								"Strong: requirements map clearly to implementation and verification steps",
								"Excellent: the plan is complete, ordered, dependency-aware, and provides \
								 explicit verification coverage for every requirement",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the implementation plan provide a concrete path from the \
							 specification to implementation, while ensuring that every \
							 requirement has corresponding verification coverage?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no plan quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no plan decision"))?;

		Ok(StageEvaluation::new(
			Stage::Plan,
			StageActor::Agent,
			started_at,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
	async fn evaluate_build(
		&self,
		session: &Path,
		intent: &str,
		spec: &str,
		plan: &str,
		tests: &str,
	) -> Result<StageEvaluation> {
		let started_at = Utc::now();

		let implementation = std::fs::read_dir(session)?
			.filter_map(|entry| entry.ok())
			.filter_map(|entry| {
				let path = entry.path();
				let name = path.file_name()?.to_string_lossy().into_owned();

				Some(if path.is_dir() {
					format!("[directory] {name}")
				} else {
					format!("[file] {name}")
				})
			})
			.collect::<Vec<_>>()
			.join("\n");

		let state = format!(
			"## User Intent\n\n{intent}\n\n\
  		 ## Specification\n\n{spec}\n\n\
  		 ## Implementation Plan\n\n{plan}\n\n\
  		 ## Test Plan\n\n{tests}\n\n\
  		 ## Repository Artifacts\n\n{implementation}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How faithfully does the implemented work satisfy the \
						 specification and implementation plan?",
							[
								"Unusable: the implementation does not meaningfully address the task",
								"Weak: substantial requirements are missing or the implementation \
							 diverges from the plan",
								"Usable: the primary requirements appear implemented but some \
							 gaps or deviations remain",
								"Strong: the implementation closely follows the specification \
							 and provides the planned functionality",
								"Excellent: the implementation comprehensively satisfies the \
							 specification and plan with no significant unexplained gaps",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the implementation appear to satisfy the specified requirements \
						 and provide the functionality described by the implementation plan?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no build quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no build decision"))?;

		Ok(StageEvaluation::new(
			Stage::Build,
			StageActor::Agent,
			started_at,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
	async fn evaluate_verification(&self, session: &Path) -> Result<StageEvaluation> {
		let started_at = Utc::now();
		let intent = SessionFile::Intent.read(&session)?;
		let spec = SessionFile::Spec.read(&session)?;
		let plan = SessionFile::Plan.read(&session)?;
		let tests = SessionFile::Tests.read(&session)?;
		let tests = SessionFile::Tests.read(&session)?;
		let evidence = SessionFile::Verification
			.read(&session)
			.unwrap_or_else(|_| String::from("No verification evidence was recorded."));

		let state = format!(
			"## User Intent\n\n{intent}\n\n\
				## Specification\n\n{spec}\n\n\
				## Test Plan\n\n{tests}\n\n\
				## Verification Evidence\n\n{evidence}"
		);

		let response = self
			.jev
			.system_one(
				state,
				[
					(
						"quality",
						Question::from(Score::new(
							"How strong is the verification evidence for establishing \
						 that the implementation satisfies the specification?",
							[
								"Unusable: there is no meaningful verification evidence",
								"Weak: some checks exist but important requirements are unverified",
								"Usable: the primary requirements have verification evidence but \
							 some gaps remain",
								"Strong: deterministic and behavioral evidence covers the \
							 requirements with only minor gaps",
								"Excellent: verification provides comprehensive, concrete evidence \
							 for every requirement and clearly establishes the intended behavior",
							],
						)),
					),
					(
						"meets_bar",
						Question::from(Noul::new(
							"Does the verification evidence provide sufficient evidence that \
						 every requirement in the specification has been satisfied?",
						)),
					),
				],
			)
			.await?;

		let quality = response
			.score("quality")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no verification quality score"))?;

		let meets_bar = response
			.noul("meets_bar")
			.ok_or_else(|| anyhow::anyhow!("JEV returned no verification decision"))?;

		Ok(StageEvaluation::new(
			Stage::Verify,
			StageActor::Sdlc,
			started_at,
			quality.score,
			quality.confidence,
			meets_bar.noul,
			vec![
				EvaluationResult {
					name: "quality".into(),
					passed: quality.score >= 0.80,
					score: quality.score,
					confidence: quality.confidence,
					explanation: String::new(),
				},
				EvaluationResult {
					name: "meets_bar".into(),
					passed: meets_bar.noul >= 0.80,
					score: meets_bar.noul,
					confidence: 1.0,
					explanation: String::new(),
				},
			],
		))
	}
}
impl SprintPipeline {
	pub fn subscribe(&self) -> broadcast::Receiver<SdlcEvent> {
		self.event_tx.subscribe()
	}
}
impl SprintPipeline {
	pub async fn run(&mut self, input_rx: &mut UnboundedReceiver<SdlcInput>) -> Result<()> {
		let mut runner = SprintRunner { pipeline: self };

		runner.run(input_rx).await
	}

	pub async fn run_simulated(&mut self, input_rx: &mut UnboundedReceiver<SdlcInput>) -> Result<()> {
		todo!("run simulated")
	}
}
impl SprintPipeline {
	//  pub async fn start(&mut self, intent: impl Into<String>) -> Result<()> {
	// let session = SdlcSession::new(intent, self.state_path.clone());
	//
	// self.session = Some(session);
	// self.last_stage = Some(Stage::Intent);
	// self.stage_attempt = 0;
	//
	// self.record_session()?;
	//
	// Ok(())
	//  }
	pub async fn start(&mut self, intent: impl Into<String>) -> Result<()> {
		let title = intent.into();

		let dir = self.create_dir(&title)?;
		self.init_templates(&dir)?;

		let session = SdlcSession::new(title, dir);

		self.session = Some(session);
		self.last_stage = Some(Stage::Intent);
		self.stage_attempt = 0;

		self.commit()?;

		Ok(())
	}
	pub fn init() -> anyhow::Result<Self> {
	dotenvy::dotenv().ok();
		let session = SpecialFile::SdlcCurrent
			.load::<SdlcSession>()
			.context("loading current SdlcSession")?;

		let evaluator = Evaluator {
			jev: TypeSafeClient::from_env()?,
		};

		let generator = Box::new(LocalGenerator {
			agent: Agent::new(),
		});

		let (event_tx, _) = broadcast::channel(256);

		let last_stage = session.as_ref().map(|session| session.stage);

		let state_path = SpecialFile::SdlcCurrent.path()?;

		Ok(Self {
			evaluator,
			generator,
			session,
			state_path,
			event_tx,
			last_stage,
			stage_attempt: 0,
		})
	}
}
impl SprintPipeline {
	async fn stage_intent(&mut self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		if session.stage != Stage::Intent {
			return Err(anyhow::anyhow!(
				"cannot execute Intent stage while at {:?}",
				session.stage
			));
		}
		self.update_progress("Intent stage completed")?;
		self.transition(Stage::Spec)?;
		Ok(())
	}
	async fn stage_spec(&mut self) -> Result<()> {
		let (stage, session_dir) = {
			let session = self
				.session
				.as_ref()
				.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
			(session.stage.clone(), session.dir.clone())
		};

		if stage != Stage::Spec {
			return Err(anyhow::anyhow!(
				"cannot execute Spec stage while at {:?}",
				stage
			));
		}

		let intent = self.read_session("intent.md")?;

		let spec = format!(
			"# Specification\n\n\
  		 ## Intent\n\n\
  		 {}\n\n\
  		 ## Requirements\n\n\
  		 - The implementation must satisfy the intent above.\n\
  		 - The implementation must be testable.\n\
  		 - Verification must provide deterministic evidence.\n",
			intent.trim()
		);
		Self::write(session_dir.join("spec.md"), spec);
		// let evaluation = self.evaluate_stage(Stage::Spec).await?;
		// self.record_evaluation(evaluation)?;
		self.update_progress("Spec stage completed")?;
		self.transition(Stage::Plan)?;
		Ok(())
	}
	async fn stage_plan(&mut self, human_input: Option<&str>) -> Result<()> {
		let (stage, session_dir) = {
			let session = self
				.session
				.as_ref()
				.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
			(session.stage.clone(), session.dir.clone())
		};
		if stage != Stage::Plan {
			return Err(anyhow::anyhow!(
				"cannot execute Plan stage while at {:?}",
				stage
			));
		}
		let intent = self.read_session("intent.md")?;
		let spec = self.read_session("spec.md")?;
		let plan = self.generate_plan(&intent, &spec).await?;
		Self::write(session_dir.join("plan.md"), plan.clone())?;
		let tests = self.generate_tests(&intent, &spec, &plan).await?;
		Self::write(session_dir.join("tests.md"), tests)?;
		// let evaluation = self.evaluate_stage(Stage::Plan).await?;
		// self.record_evaluation(evaluation)?;
		self.update_progress("Plan and test plan generated")?;
		self.transition(Stage::Build)?;
		Ok(())
	}
	async fn stage_build(&mut self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let context = AgentContext::from_session(session)?;
		self.update_progress("Build started")?;
		let task = context.task.clone();
		self.transition(Stage::Verify)?;
		Ok(())
	}
	async fn stage_verify(&mut self) -> Result<Verification> {
		self.update_progress("Verification started")?;
		let checks = self.run_checks().await?;
		let evaluations = self.evaluate(&checks).await?;
		let verification = Verification {
			passed: self.is_passing(&checks, &evaluations),
			checks,
			evaluations,
		};
		self.update_progress(&format!(
			"Verification completed: passed={}",
			verification.passed
		))?;

		Ok(verification)
	}

	async fn stage_deploy(&mut self) -> Result<()> {
		todo!("sdlc deploy")
	}

	async fn stage_maintain(&mut self) -> Result<()> {
		todo!("sdlc maintain")
	}

	async fn stage_complete(&mut self) -> Result<RunControl> {
		todo!("WOW DONE!")
	}

	pub fn cancel(&mut self) {
		todo!("cancel")
	}

	pub fn fail(&mut self, _outcome: StageOutcome) -> Result<()> {
		todo!("fail")
	}
	async fn stage_execute(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<StageExecution> {
		let started_at = Utc::now();

		let result = match stage {
			Stage::Intent => {
				self.stage_intent().await?;
				StageResult::Intent
			}
			Stage::Spec => {
				self.stage_spec().await?;
				StageResult::Spec
			}
			Stage::Plan => {
				let input = pending_input.take();
				self
					.stage_plan(input.as_ref().and_then(SdlcInput::text))
					.await?;
				StageResult::Plan
			}
			Stage::Build => {
				self.stage_build().await?;
				StageResult::Build
			}
			Stage::Verify => {
				let verification = self.stage_verify().await?;
				StageResult::Verification(verification)
			}
			Stage::Complete => StageResult::Complete,
			Stage::SprintCompleted => StageResult::SprintCompleted,
			Stage::Deploy | Stage::Maintain => {
				return Err(anyhow!("stage {stage:?} not implemented"));
			}
		};
		Ok(StageExecution {
			stage,
			attempt,
			started_at,
			completed_at: Utc::now(),
			result,
		})
	}
	async fn execute(&mut self, stage: enums::Stage) -> Result<StageExecution> {
		todo!()
	}
	pub fn decide(&self, outcome: &StageOutcome) -> Result<StageDecision> {
		let decision = match outcome {
			StageOutcome::Complete { execution, .. } => {
				if execution.stage == enums::Stage::Complete {
					StageDecision::Complete
				} else {
					StageDecision::Continue
				}
			}
			StageOutcome::NeedsRevision { .. } => StageDecision::Revise,
			StageOutcome::ExecutionFailed { .. } | StageOutcome::EvaluationFailed { .. } => {
				StageDecision::AwaitHuman
			}
		};

		Ok(decision)
	}
	async fn apply(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		outcome: StageOutcome,
		decision: StageDecision,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		match decision {
			StageDecision::Continue => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;

				self.transition(next)?;

				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});

				Ok(RunControl::Continue)
			}

			StageDecision::Retry => {
				// self.handle_retry(stage, attempt).await?;
				Ok(RunControl::Continue)
			}

			StageDecision::Revise => {
				// self
				// 	.handle_revision(stage, attempt, outcome, input_rx, pending_input)
				// 	.await?;

				Ok(RunControl::Continue)
			}

			StageDecision::AwaitHuman => {
				self
					.wait_for_intervention(stage, attempt, String::from("Evaluator Decision"), input_rx)
					.await?;
				//
				Ok(RunControl::Continue)
			}

			StageDecision::Complete => {
				self.stage_complete().await?;
				Ok(RunControl::Exit)
			}

			StageDecision::Fail => {
				self.fail(outcome)?;
				Ok(RunControl::Exit)
			}
		}
	}

	async fn wait_for_intervention(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		reason: String,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
	) -> Result<Intervention> {
		self.emit(SdlcEvent::InterventionRequired {
			stage,
			attempt,
			reason,
		});
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});
		let input = input_rx
			.recv()
			.await
			.ok_or_else(|| anyhow!("SDLC input channel closed"))?;

		match input {
			SdlcInput::Human(string) => {
				self.emit(SdlcEvent::HumanInput {
					stage,
					input: string.clone(),
				});

				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "human input provided".into(),
				});

				Ok(Intervention::Human(string))
			}
			SdlcInput::Retry => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "retry".into(),
				});

				Ok(Intervention::Retry)
			}
			SdlcInput::ProvideContext(context) => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "context provided".into(),
				});

				Ok(Intervention::ProvideContext(context))
			}
			SdlcInput::Reviewed => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "reviewed".into(),
				});

				Ok(Intervention::Reviewed)
			}
			SdlcInput::Abort => {
				self.emit(SdlcEvent::InterventionResolved {
					stage,
					action: "abort".into(),
				});

				Ok(Intervention::Abort)
			}
		}
	}
}
impl SprintPipeline {
	fn next_attempt(&mut self, stage: Stage) -> Attempt {
		if self.last_stage != Some(stage) {
			self.last_stage = Some(stage);
			self.stage_attempt = 1;
		} else {
			self.stage_attempt += 1;
		}

		Attempt {
			stage,
			number: self.stage_attempt,
		}
	}
	pub fn stage(&self) -> Option<Stage> {
		self.last_stage
	}

	pub fn stage_attempt(&self) -> u32 {
		self.stage_attempt
	}
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Attempt {
	pub stage: Stage,
	pub number: u32,
}
impl Attempt {
	pub fn new() -> Self {
		Self {
			stage: Stage::Intent,
			number: 0,
		}
	}
}
impl std::fmt::Display for Attempt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} attempt {}", self.stage, self.number)
	}
}
impl SprintPipeline {
	fn commit(&mut self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow!("no active SDLC session"))?;

		FS::save(SpecialFile::SdlcCurrent.path()?, session)?;

		Ok(())
	}
	fn create_dir(&self, title: &str) -> Result<PathBuf> {
		eprintln!("CREATE DIR");
		eprintln!("  title = {title:?}");
		// eprintln!("  dir   = {dir:?}");
		eprintln!("  cwd   = {:?}", std::env::current_dir()?);
		let sessions_dir = FS::ensure_dir(SpecialFile::SessionsDir.path()?)?;
		let date = Local::now().format("%Y-%m-%d");
		let dir = sessions_dir.join(format!("{date}.{title}"));
		FS::ensure_dir(&dir)?;
		Ok(dir)
	}
	fn dir(&self) -> Result<&Path> {
		self
			.session
			.as_ref()
			.map(|session| session.dir.as_path())
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))
	}
	fn init_templates(&self, dir: &Path) -> Result<()> {
		let template_dir = SpecialFile::AiTemplateDir.path()?;
		for file in [
			SessionFile::Intent,
			SessionFile::Spec,
			SessionFile::Plan,
			SessionFile::Progress,
		] {
			let source = template_dir.join(file.name());
			let destination = file.path(dir);
			let contents = if FS::exists(&source) {
				FS::read(source)?
			} else {
				format!("# {}\n\n", file.name())
			};
			FS::write(destination, contents)?;
		}
		Ok(())
	}
	fn is_passing(&self, checks: &[CheckResult], evaluations: &[EvaluationResult]) -> bool {
		checks.iter().all(|check| check.passed)
			&& evaluations.iter().all(|evaluation| evaluation.passed)
	}
	fn emit(&self, event: SdlcEvent) {
		let _ = self.event_tx.send(event);
	}
	async fn generate_plan(&self, intent: &str, spec: &str) -> Result<String> {
		let prompt = prompt::plan_gen(intent, spec);
		self.generator.generate(&prompt).await
	}
	async fn generate_tests(&self, intent: &str, spec: &str, plan: &str) -> Result<String> {
		let prompt = prompt::tests_gen(intent, spec, plan);
		self.generator.generate(&prompt).await
	}
	async fn evaluate(&self, checks: &[CheckResult]) -> Result<Vec<EvaluationResult>> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let intent = self.read_session("intent.md")?;
		let spec = self.read_session("spec.md");
		let plan = self.read_session("plan.md");
		let progress = self.read_session("progress.md");
		Ok(vec![])
	}

	fn persist(&self) -> Result<()> {
		FS::save(&self.state_path, &self.session)
	}
	fn read_session(&self, name: &str) -> Result<String> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		read_from_session(name, session)
	}
	fn record_outcome(&mut self, outcome: &StageOutcome) -> Result<()> {
		let session = self
			.session
			.as_mut()
			.ok_or_else(|| anyhow!("no active SDLC session"))?;

		let (stage, attempt, status, actor, started_at, completed_at, evaluation) = match outcome {
			StageOutcome::Complete {
				execution,
				evaluation,
			} => (
				execution.stage,
				execution.attempt,
				StageStatus::Completed,
				evaluation.actor.clone(),
				execution.started_at,
				execution.completed_at,
				Some(evaluation.clone()),
			),

			StageOutcome::NeedsRevision {
				execution,
				evaluation,
			} => (
				execution.stage,
				execution.attempt,
				StageStatus::NeedsRevision,
				evaluation.actor.clone(),
				execution.started_at,
				execution.completed_at,
				Some(evaluation.clone()),
			),

			StageOutcome::ExecutionFailed {
				stage,
				attempt,
				error: _,
			} => (
				*stage,
				*attempt,
				StageStatus::Failed,
				StageActor::Sdlc,
				Utc::now(),
				Utc::now(),
				None,
			),

			StageOutcome::EvaluationFailed {
				execution,
				error: _,
			} => (
				execution.stage,
				execution.attempt,
				StageStatus::EvaluationFailed,
				StageActor::Sdlc,
				execution.started_at,
				execution.completed_at,
				None,
			),
		};

		let record = StageRecord {
			stage,
			attempt,
			status,
			actor,
			started_at,
			completed_at: Some(completed_at),
			evaluation,
		};

		session.stages.push(record);
		session.updated_at = Utc::now();

		self.persist()
	}
	fn record_stage_outcome(&mut self, outcome: &StageOutcome) -> Result<()> {
		self.record_outcome(outcome)
	}
	fn record_session(&self) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;

		let index_path = SpecialFile::SessionsIndex.path()?;

		let mut sessions: Vec<SdlcSession> = FS::load(&index_path)?.unwrap_or_default();

		sessions.retain(|existing| existing.id != session.id);
		sessions.push(session.clone());

		FS::save(index_path, &sessions)?;

		Ok(())
	}
	fn record_evaluation(&mut self, evaluation: StageEvaluation) -> Result<()> {
		let session = self
			.session
			.as_mut()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let status = if evaluation.passed {
			StageStatus::Completed
		} else {
			StageStatus::NeedsRevision
		};
		let record = StageRecord {
			actor: evaluation.actor.clone(),
			attempt: Attempt::new(),
			completed_at: Some(Utc::now()),
			evaluation: Some(evaluation.clone()),
			stage: evaluation.stage.clone(),
			started_at: evaluation.started_at,
			status,
		};
		session.stages.push(record);
		session.updated_at = Utc::now();
		self.persist()
	}
	async fn resume(&mut self) -> Result<()> {
		if self.session.is_none() {
			return Err(anyhow::anyhow!("no active SDLC session to resume"));
		}

		self.update_progress("SDLC session resumed")?;
		self.persist()?;

		Ok(())
	}
	fn retry(&mut self, _stage: enums::Stage) -> Result<()> {
		Ok(())
	}
	async fn run_checks(&self) -> Result<Vec<CheckResult>> {
		let checks = [
			("cargo check", vec!["cargo", "check"]),
			("cargo test", vec!["cargo", "test"]),
			(
				"cargo clippy",
				vec!["cargo", "clippy", "--", "-D", "warnings"],
			),
			("cargo fmt", vec!["cargo", "fmt", "--", "--check"]),
		];

		let mut results = Vec::with_capacity(checks.len());

		for (name, command) in checks {
			let result = Command::new(command[0])
				.args(&command[1..])
				.current_dir(env!("CARGO_MANIFEST_DIR"))
				.output()
				.map_err(|error| anyhow::anyhow!("failed to run verification check `{name}`: {error}"))?;

			let stdout = String::from_utf8_lossy(&result.stdout);
			let stderr = String::from_utf8_lossy(&result.stderr);

			let output = if stderr.is_empty() {
				stdout.into_owned()
			} else if stdout.is_empty() {
				stderr.into_owned()
			} else {
				format!("{stdout}\n{stderr}")
			};

			results.push(CheckResult {
				name: name.to_string(),
				passed: result.status.success(),
				output: Some(output),
			});

			if !result.status.success() {
				break;
			}
		}

		Ok(results)
	}
	fn transition(&mut self, next: enums::Stage) -> Result<()> {
		let session = self
			.session
			.as_mut()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let valid = matches!(
			(&session.stage, &next),
			(Stage::Intent, Stage::Spec)
				| (Stage::Spec, Stage::Plan)
				| (Stage::Plan, Stage::Build)
				| (Stage::Build, Stage::Verify)
				| (Stage::Verify, Stage::Build)
				| (Stage::Verify, Stage::Complete)
		);

		if !valid {
			return Err(anyhow::anyhow!(
				"invalid SDLC transition: {:?} -> {:?}",
				session.stage,
				next
			));
		}
		session.stage = next;
		session.updated_at = Utc::now();
		self.persist()?;
		self.record_session()?;
		Ok(())
	}
	fn update_progress(&self, message: &str) -> Result<()> {
		let session = self
			.session
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("no active SDLC session"))?;
		let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
		let entry = format!("\n## {timestamp}\n\n{message}\n");
		SessionFile::Progress.append(&session.dir, entry)?;
		Ok(())
	}
	fn write(path: PathBuf, contents: String) -> Result<()> {
		Ok(std::fs::write(path, contents)?)
	}
}
impl PipelineRuntime {
	pub fn new(stage: enums::Stage) -> Self {
		Self {
			activity: vec![],
			attempt: 0,
			confidence: None,
			history: Vec::new(),
			message: None,
			passed: None,
			phase: SdlcPhase::Starting,
			score: None,
			stage,
			stage_started_at: Instant::now(),
			started_at: Instant::now(),
			total_agent_calls: 0,
			total_tokens: 0,
		}
	}
}
impl SdlcSession {
	fn created_at_readable(&self) -> String {
		self.created_at.format(FMT_HUMAN_READABLE).to_string()
	}
	fn updated_at_readable(&self) -> String {
		self.updated_at.format(FMT_HUMAN_READABLE).to_string()
	}
	fn create_readable(&self) -> String {
		let current = Utc::now();
		current.format(FMT_HUMAN_READABLE).to_string()
	}
}
impl SdlcView {
	pub fn new(stage: enums::Stage) -> Self {
		Self {
			input_active: false,
			input: String::new(),
			paused: false,
			show_logs: false,
			runtime: PipelineRuntime::new(stage),
		}
	}

	pub fn begin_input(&mut self) {
		self.input_active = true;
		self.input.clear();
	}
	pub fn end_input(&mut self) {
		self.input_active = false;
		self.input.clear();
	}
	pub fn is_input_active(&self) -> bool {
		self.input_active
	}
	pub fn toggle_pause(&mut self) {
		self.paused = !self.paused;
	}
	pub fn toggle_logs(&mut self) {
		self.show_logs = !self.show_logs;
	}
	pub fn handle_input_key(
		&mut self,
		key: KeyEvent,
		input_tx: &UnboundedSender<SdlcInput>,
	) -> anyhow::Result<()> {
		match key.code {
			KeyCode::Char(c) => {
				self.input.push(c);
			}

			KeyCode::Backspace => {
				self.input.pop();
			}
			KeyCode::Enter => {
				let input = std::mem::take(&mut self.input);
				input_tx.send(SdlcInput::Human(input))?;
				self.input_active = false;
			}

			KeyCode::Esc => {
				self.input_active = false;
				self.input.clear();
			}

			_ => {}
		}

		Ok(())
	}
	pub fn render(frame: &mut Frame<'_>, view: &SdlcView) {
		let area = frame.area();
		frame.render_widget(Clear, area);
		let chunks = RatatuiLayout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Length(2),
				Constraint::Length(3),
				Constraint::Min(8),
				Constraint::Length(3),
			])
			.split(area);

		ui::stepper(frame, view, chunks[1]);
		let body = ui::body(chunks[2]);
		ui::left_stage_panel(frame, view, body[0]);
		ui::right_activity_panel(frame, view, body[2]);
		ui::footer(frame, view, chunks[3]);
	}
}
impl SdlcView {
	pub fn apply(&mut self, event: SdlcEvent) {
		// 		match event {
		// 			SdlcEvent::RunStarted {
		// 				run_id,
		// 				stage,
		// 				..
		// 			} => {
		// 				self.runtime.run_id = run_id;
		// 				self.runtime.stage = stage;
		// 				self.runtime.attempt = 0;
		// 				self.runtime.phase = PipelinePhase::Running;
		// 				self.runtime.started_at = Some(Instant::now());
		// 				self.runtime.stage_started_at = Some(Instant::now());
		// 				self.runtime.message = "Run started".into();
		// 			}
		//
		// 			SdlcEvent::StageStarted {
		// 				stage,
		// 				attempt,
		// 				..
		// 			} => {
		// 				self.runtime.stage = stage;
		// 				self.runtime.attempt = attempt;
		// 				self.runtime.phase = PipelinePhase::Running;
		// 				self.runtime.stage_started_at = Some(Instant::now());
		// 				self.runtime.message =
		// 					format!("Running {stage:?}");
		// 			}
		//
		// 			SdlcEvent::ExecutionStarted { .. } => {
		// 				self.runtime.phase = PipelinePhase::Executing;
		// 				self.runtime.message = "Executing".into();
		// 			}
		//
		// 			SdlcEvent::ExecutionCompleted { .. } => {
		// 				self.runtime.phase = PipelinePhase::Evaluating;
		// 				self.runtime.message = "Evaluating".into();
		// 			}
		//
		// 			SdlcEvent::Evaluated {
		// 				score,
		// 				confidence,
		// 				..
		// 			} => {
		// 				self.runtime.score = Some(score);
		// 				self.runtime.confidence = Some(confidence);
		// 				self.runtime.phase = PipelinePhase::Evaluating;
		// 				self.runtime.message = format!(
		// 					"Evaluation: {:.2} (confidence {:.2})",
		// 					score,
		// 					confidence,
		// 				);
		// 			}
		//
		// 			SdlcEvent::AwaitingHuman { .. } => {
		// 				self.runtime.phase = PipelinePhase::AwaitingHuman;
		// 				self.runtime.message = "Awaiting human input".into();
		// 			}
		//
		// 			SdlcEvent::Retrying {
		// 				stage,
		// 				attempt,
		// 				..
		// 			} => {
		// 				self.runtime.stage = stage;
		// 				self.runtime.attempt = attempt;
		// 				self.runtime.phase = PipelinePhase::Retrying;
		// 				self.runtime.message =
		// 					format!("Retrying {stage:?} (attempt {attempt})");
		// 			}
		//
		// 			SdlcEvent::StageCompleted {
		// 				stage,
		// 				..
		// 			} => {
		// 				self.runtime.stage = stage;
		// 				self.runtime.phase = PipelinePhase::Completed;
		// 				self.runtime.message =
		// 					format!("{stage:?} complete");
		// 			}
		//
		// 			SdlcEvent::RunCompleted { .. } => {
		// 				self.runtime.phase = PipelinePhase::Completed;
		// 				self.runtime.message = "SDLC complete".into();
		// 			}
		//
		// 			SdlcEvent::Failed {
		// 				stage,
		// 				error,
		// 				..
		// 			} => {
		// 				if let Some(stage) = stage {
		// 					self.runtime.stage = stage;
		// 				}
		//
		// 				self.runtime.phase = PipelinePhase::Failed;
		// 				self.runtime.message = error.clone();
		//
		// 				self.runtime.error = Some(error);
		// 			}
		//
		// 			// Anything that is purely informational should generally
		// 			// go into the log/history rather than mutate the primary
		// 			// runtime state.
		// 			event => {
		// 				self.runtime.events.push(event);
		// 			}
		// 		}
	}
}

impl SdlcInput {
	pub fn text(&self) -> Option<&str> {
		match self {
			Self::Human(value) | Self::ProvideContext(value) => Some(value),
			Self::Abort | Self::Retry | Self::Reviewed => None,
		}
	}
}
impl enums::Stage {
	pub fn next(&self) -> Option<Self> {
		match self {
			Self::Intent => Some(Self::Spec),
			Self::Spec => Some(Self::Plan),
			Self::Plan => Some(Self::Build),
			Self::Build => Some(Self::Verify),
			Self::Verify => Some(Self::Complete),
			Self::Deploy => Some(Self::Maintain),
			Self::Maintain => Some(Self::Complete),
			Self::Complete => None,
			Self::SprintCompleted => None,
		}
	}
	pub fn is_before(self, other: enums::Stage) -> bool {
		let rank = |stage: enums::Stage| match stage {
			Stage::Intent => 0,
			Stage::Spec => 1,
			Stage::Plan => 2,
			Stage::Build => 3,
			Stage::Verify => 4,
			Stage::Deploy => 5,
			Stage::Maintain => 6,
			Stage::Complete => 7,
			Stage::SprintCompleted => 8,
		};
		rank(self) < rank(other)
	}
}
impl StageEvaluation {
	fn new(
		stage: Stage,
		actor: StageActor,
		started_at: DateTime<Utc>,
		score: f64,
		confidence: f64,
		meets_bar: f64,
		evaluations: Vec<EvaluationResult>,
	) -> Self {
		Self {
			stage,
			actor,
			started_at,
			completed_at: Utc::now(),
			score,
			confidence,
			passed: meets_bar >= 0.80 && confidence >= 0.70,
			evaluations,
		}
	}
}
impl Step {
	pub const ALL: &'static [Self] = &[
		Self::Init,
		Self::Intent,
		Self::Spec,
		Self::Plan,
		Self::Build,
		Self::Verify,
		Self::Deploy,
		Self::Maintain,
		Self::Complete,
	];
}
#[async_trait::async_trait]
impl traits::Runner for SprintRunner<'_> {
	type Context = UnboundedReceiver<SdlcInput>;
	type Output = ();

	async fn run(&mut self, input_rx: &mut Self::Context) -> Result<Self::Output> {
		let mut pending_input = None;

		self.emit(SdlcEvent::RunStarted);

		loop {
			let stage = self.load_state()?;
			// let stage = state.latest_incomplete_stage()?;
			let attempt = self.begin_attempt(stage);
			let outcome = self.run_stage(stage, attempt, &mut pending_input).await?;
			self.persist_outcome(&outcome)?;
			let decision = self.decide(&outcome).await?;
			let control = self
				.apply(outcome, decision, input_rx, &mut pending_input)
				.await?;

			match control {
				RunControl::Continue => {
					// Intentionally loop back through persistence.
					continue;
				}
				RunControl::Exit => return Ok(()),
			}
		}
	}

	fn cancel(&mut self) {
		self.pipeline.cancel();
	}

	fn is_running(&self) -> bool {
		true
		// self.pipeline.is_running()
	}
}
impl SprintRunner<'_> {
	fn load_state(&mut self) -> Result<enums::Stage> {
		todo!("load_state")
	}

	fn persist_outcome(&mut self, _outcome: &StageOutcome) -> Result<()> {
		Ok(())
	}

	async fn resolve_execution(&mut self, _execution: StageExecution) -> Result<StageOutcome> {
		todo!("resolve_execution")
	}
}
impl SprintRunner<'_> {
	fn begin_attempt(&mut self, stage: Stage) -> Attempt {
		self.pipeline.next_attempt(stage)
	}
	async fn run_stage(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<StageOutcome> {
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Executing,
		});

		let execution = match self
			.pipeline
			.stage_execute(stage, attempt, pending_input)
			.await
		{
			Ok(execution) => {
				self.emit(SdlcEvent::ExecutionComplete { stage });
				execution
			}

			Err(error) => {
				return Ok(StageOutcome::ExecutionFailed {
					stage,
					attempt,
					error,
				});
			}
		};

		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Evaluating,
		});

		self.resolve_execution(execution).await
	}
	async fn decide(&mut self, outcome: &StageOutcome) -> Result<StageDecision> {
		self.pipeline.decide(outcome)
	}

	fn transition(&mut self, next: enums::Stage) -> Result<()> {
		self.pipeline.transition(next)
	}

	fn retry(&mut self, stage: enums::Stage) -> Result<()> {
		self.pipeline.retry(stage)
	}

	// fn emit(&self, event: SdlcEvent) {
	// 	self.pipeline.emit(event);
	// }

	fn emit(&self, event: SdlcEvent) {
		let _ = self.pipeline.event_tx.send(event);
	}

	async fn wait_for_intervention(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		reason: String,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
	) -> Result<Intervention> {
		self
			.pipeline
			.wait_for_intervention(stage, attempt, reason, input_rx)
			.await
	}

	async fn apply(
		&mut self,
		outcome: StageOutcome,
		decision: StageDecision,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		let stage = outcome.stage();
		let attempt = outcome.attempt();

		match decision {
			StageDecision::Continue => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;

				self.transition(next)?;

				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});

				Ok(RunControl::Continue)
			}

			StageDecision::Retry => {
				self.handle_retry(stage, attempt).await?;
				Ok(RunControl::Continue)
			}

			StageDecision::Revise => {
				self
					.handle_revision(stage, attempt, outcome, input_rx, pending_input)
					.await?;

				Ok(RunControl::Continue)
			}

			StageDecision::AwaitHuman => {
				self
					.handle_human_intervention(stage, attempt, input_rx, pending_input)
					.await?;

				Ok(RunControl::Continue)
			}

			StageDecision::Complete => {
				self.pipeline.stage_complete().await?;
				Ok(RunControl::Exit)
			}

			StageDecision::Fail => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "stage failed".into(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
	async fn handle_human_intervention(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<()> {
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});

		match self
			.wait_for_intervention(
				stage,
				attempt,
				String::from("Human intervention required"),
				input_rx,
			)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input));
				self.pipeline.retry(stage)?;
			}

			Intervention::Retry => {
				self.pipeline.retry(stage)?;
			}

			Intervention::ProvideContext(context) => {
				*pending_input = Some(SdlcInput::ProvideContext(context));
				self.pipeline.retry(stage)?;
			}

			Intervention::Reviewed => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;

				self.transition(next)?;

				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});
			}
		}

		Ok(())
	}
	async fn handle_retry(&mut self, stage: enums::Stage, attempt: Attempt) -> Result<()> {
		let next_attempt = Attempt {
			stage: attempt.stage,
			number: attempt.number + 1,
		};
		self.emit(SdlcEvent::StageRetrying {
			stage: next_attempt.stage,
			number: next_attempt.number,
		});
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::Retrying,
		});
		self.pipeline.retry(stage)?;
		Ok(())
	}

	async fn handle_revision(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		_outcome: StageOutcome,
		input_rx: &mut UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<()> {
		// self.emit(SdlcEvent::PhaseChanged {
		// 	phase: SdlcPhase::Revising,
		// });

		match self
			.wait_for_intervention(
				stage,
				attempt,
				String::from("Stage needs revision"),
				input_rx,
			)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input));
				self.pipeline.retry(stage)?;
			}

			Intervention::Retry => {
				self.pipeline.retry(stage)?;
			}
			Intervention::ProvideContext(context) => {
				*pending_input = Some(SdlcInput::ProvideContext(context));
				self.pipeline.retry(stage)?;
			}

			Intervention::Reviewed => {
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;

				self.transition(next)?;

				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});
			}
		}

		Ok(())
	}

	async fn handle_failure_execution(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		error: anyhow::Error,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		self.emit(SdlcEvent::ExecutionFailed {
			stage,
			attempt,
			error: error.to_string(),
		});
		if attempt.number < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				number: attempt.number + 1,
			});
			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Retrying,
			});
			self.pipeline.retry(stage)?;
			return Ok(RunControl::Continue);
		}
		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});

		match self
			.wait_for_intervention(stage, attempt, String::from("Execution Failure"), input_rx)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.as_str().to_owned()));
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: error.to_string(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
	async fn handle_failure_evaluation(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		error: anyhow::Error,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		self.emit(SdlcEvent::EvaluationFailed {
			stage,
			error: error.to_string(),
		});

		if attempt.number < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				number: attempt.number + 1,
			});

			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Retrying,
			});

			self.pipeline.retry(stage)?;
			return Ok(RunControl::Continue);
		}

		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});

		match self
			.wait_for_intervention(stage, attempt, String::from("Evaluation Failure"), input_rx)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.to_string()));
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				// If ProvideContext eventually becomes stage input,
				// this is where it should be stored.
				let _ = context;

				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
	async fn handle_failure_of_quality(
		&mut self,
		stage: enums::Stage,
		attempt: Attempt,
		input_rx: &mut tokio::sync::mpsc::UnboundedReceiver<SdlcInput>,
		pending_input: &mut Option<SdlcInput>,
	) -> Result<RunControl> {
		if attempt.number < MAX_STAGE_ATTEMPTS {
			self.emit(SdlcEvent::StageRetrying {
				stage,
				number: attempt.number + 1,
			});

			self.emit(SdlcEvent::PhaseChanged {
				phase: SdlcPhase::Retrying,
			});

			self.pipeline.retry(stage)?;

			return Ok(RunControl::Continue);
		}

		self.emit(SdlcEvent::PhaseChanged {
			phase: SdlcPhase::AwaitingHuman,
		});

		match self
			.wait_for_intervention(
				stage,
				attempt,
				String::from("Quality Error (Needs Revision)"),
				input_rx,
			)
			.await?
		{
			Intervention::Human(input) => {
				*pending_input = Some(SdlcInput::Human(input.to_string()));
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Retry => {
				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::ProvideContext(context) => {
				let _ = context;

				self.pipeline.retry(stage)?;

				Ok(RunControl::Continue)
			}

			Intervention::Reviewed => {
				// "Reviewed" means the human explicitly accepts
				// the current artifact despite JEV not passing it.
				let next = stage
					.next()
					.ok_or_else(|| anyhow!("Stage {stage:?} has no next stage"))?;

				self.transition(next)?;

				self.emit(SdlcEvent::StageTransitioned {
					from: stage,
					to: next,
				});

				Ok(RunControl::Continue)
			}

			Intervention::Abort => {
				self.emit(SdlcEvent::Failed {
					stage: Some(stage),
					error: "aborted by user".into(),
				});

				Ok(RunControl::Exit)
			}
		}
	}
}
impl StageOutcome {
	fn stage(&self) -> enums::Stage {
		match self {
			Self::Complete { execution, .. }
			| Self::NeedsRevision { execution, .. }
			| Self::EvaluationFailed { execution, .. } => execution.stage,

			Self::ExecutionFailed { stage, .. } => *stage,
		}
	}

	fn attempt(&self) -> Attempt {
		match self {
			Self::Complete { execution, .. }
			| Self::NeedsRevision { execution, .. }
			| Self::EvaluationFailed { execution, .. } => execution.attempt,

			Self::ExecutionFailed { attempt, .. } => *attempt,
		}
	}
}

pub struct TerminalGuard;

impl Drop for TerminalGuard {
	fn drop(&mut self) {
		let _ = disable_raw_mode();
		let mut stdout = stdout();
		let _ = execute!(stdout, LeaveAlternateScreen);
		let _ = execute!(stdout, cursor::Show);
	}
}

const TODO: &'static str = r#"
  - Question prompt
  - Add progressive disclosure
"#;
