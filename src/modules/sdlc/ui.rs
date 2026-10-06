use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::{
	agent_event::RuntimeEvent,
	model::{
		AgentTask,
		agent::{Agent, AgentContext},
		resolver::*,
		task::TaskResult,
	},
	prelude::{structs as ext_structs, *},
	sdlc::{format_elapsed, r#struct::AiView},
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
pub fn header(frame: &mut Frame<'_>, view: &AiView, area: Rect) {
	let elapsed = format_elapsed(view.runtime.time_started.elapsed());
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
pub fn stepper(frame: &mut Frame<'_>, view: &AiView, area: Rect) {
	let stages = [
		Stage::Intent,
		Stage::Spec,
		Stage::Plan,
		Stage::Build,
		Stage::QA,
		Stage::Complete,
	];

	let current = view.runtime.stage;

	let current_style = match view.runtime.phase {
		Phase::Failed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),

		Phase::Retrying => Style::default()
			.fg(Color::Yellow)
			.add_modifier(Modifier::BOLD),

		Phase::AwaitingHuman => Style::default()
			.fg(Color::Magenta)
			.add_modifier(Modifier::BOLD),

		_ => Style::default()
			.fg(Color::Yellow)
			.add_modifier(Modifier::BOLD),
	};

	let spinner_text = spinner(view.runtime.stage_time_started.elapsed());

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
pub fn left_stage_panel(frame: &mut Frame<'_>, view: &AiView, area: Rect) {
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
		Phase::Executing => {
			let frame = spinner(runtime.stage_time_started.elapsed());

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
		Phase::Retrying => Line::from(vec![
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
		Phase::Evaluating => Line::from(vec![
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
		Phase::AwaitingHuman => Line::from(vec![
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
		Phase::Failed => Line::from(vec![
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
		Phase::Starting => Line::from(vec![
			Span::styled("· ", Style::default().fg(Color::Yellow)),
			Span::styled(
				format!("{:?}", runtime.stage),
				Style::default()
					.fg(Color::White)
					.add_modifier(Modifier::BOLD),
			),
		]),
		Phase::Completed => Line::from(vec![
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
				format_elapsed(runtime.stage_time_started.elapsed()),
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
pub fn right_activity_panel(frame: &mut Frame<'_>, view: &AiView, area: Rect) {
	let runtime = &view.runtime;
	let spinner = spinner(runtime.stage_time_started.elapsed());
	let stage_style = Style::default()
		.fg(Color::White)
		.add_modifier(Modifier::BOLD);
	let attempt_style = Style::default().fg(Color::DarkGray);
	let phase_style = phase_style(runtime.phase);
	let phase_label = match runtime.phase {
		Phase::Starting => "Starting",
		Phase::Executing => "Executing",
		Phase::Evaluating => "Evaluating",
		Phase::Retrying => "Retrying",
		Phase::AwaitingHuman => "Awaiting input",
		Phase::Completed => "Completed",
		Phase::Failed => "Failed",
	};
	let mut lines = Vec::new();
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
pub fn footer(frame: &mut Frame<'_>, view: &AiView, area: Rect) {
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
		SdlcEvent::Activity {
			stage,
			attempt,
			message,
		} => Line::from(Span::styled(
			format!("↳ {message}"),
			Style::default().fg(Color::Gray),
		)),
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
pub fn phase_style(phase: Phase) -> Style {
	match phase {
		Phase::Starting => Style::default()
			.fg(Color::Yellow)
			.add_modifier(Modifier::BOLD),

		Phase::Executing => Style::default()
			.fg(Color::Yellow)
			.add_modifier(Modifier::BOLD),

		Phase::Evaluating => Style::default()
			.fg(Color::Cyan)
			.add_modifier(Modifier::BOLD),

		Phase::Retrying => Style::default()
			.fg(Color::Yellow)
			.add_modifier(Modifier::BOLD),

		Phase::AwaitingHuman => Style::default()
			.fg(Color::Magenta)
			.add_modifier(Modifier::BOLD),

		Phase::Completed => Style::default()
			.fg(Color::Green)
			.add_modifier(Modifier::BOLD),

		Phase::Failed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
	}
}
pub fn spinner(elapsed: std::time::Duration) -> &'static str {
	const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
	let index = (elapsed.as_millis() / 100) as usize % FRAMES.len();
	FRAMES[index]
}

use std::{
	io::{self, Write},
	sync::Arc,
	time::Duration,
};

use chrono::Local;
use std::io::IsTerminal;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crossterm::{
	cursor, execute,
	terminal::{self, ClearType},
};

pub struct StatusBar {
	running: bool,
}

impl StatusBar {
	pub fn new() -> Self {
		Self { running: true }
	}

	pub fn draw(&self) -> io::Result<()> {
		let (_, height) = terminal::size()?;
		let mut stdout = io::stdout();

		// Save current cursor position.
		execute!(stdout, cursor::SavePosition)?;

		// Leave one blank row above the clock.
		execute!(
			stdout,
			cursor::MoveTo(0, height.saturating_sub(2)),
			terminal::Clear(ClearType::CurrentLine),
		)?;

		execute!(
			stdout,
			cursor::MoveTo(0, height.saturating_sub(1)),
			terminal::Clear(ClearType::CurrentLine),
		)?;

		let time = chrono::Local::now().format("%H:%M:%S").to_string();

		let width = terminal::size()?.0 as usize;

		execute!(
			stdout,
			cursor::MoveTo(
				width.saturating_sub(time.len()) as u16,
				height.saturating_sub(1)
			),
		)?;

		print!("{time}");

		// Put the cursor back where normal output expects it.
		execute!(stdout, cursor::RestorePosition)?;

		stdout.flush()
	}

	pub fn clear(&self) -> io::Result<()> {
		let (_, height) = terminal::size()?;
		let mut stdout = io::stdout();

		execute!(
			stdout,
			cursor::SavePosition,
			cursor::MoveTo(0, height.saturating_sub(2)),
			terminal::Clear(ClearType::FromCursorDown),
			cursor::RestorePosition,
		)?;

		stdout.flush()
	}
}

pub struct TerminalUi {
	status: StatusBar,
}

impl TerminalUi {
	pub fn new() -> Self {
		Self {
			status: StatusBar::new(),
		}
	}
	pub fn println(&self, message: impl AsRef<str>) -> io::Result<()> {
		self.status.clear()?;

		println!("{}", message.as_ref());

		self.status.draw()?;

		Ok(())
	}
}
// let ui = TerminalUi::new();
// 
// ui.println(">>> starting runtime")?;
// 
// if resume_from_build {
//     ui.println(">>> resuming from Build")?;
// 
//     runtime
//         .resume_from(Stage::Build, &mut input_rx)
//         .await
//         .context("runtime.resume_from(Build)")?;
// } else {
//     runtime
//         .run(&mut input_rx)
//         .await
//         .context("runtime.run")?;
// }
// 
// ui.println(">>> runtime finished")?;