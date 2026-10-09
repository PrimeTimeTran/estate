use anyhow::{Context, anyhow};
use crossterm::{
	cursor,
	event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};
pub use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
	widgets::Clear,
};
use std::{
	io::Stdout,
	path::{Path, PathBuf},
	process::Command,
};
use tokio::time::{Duration, sleep};

use crate::{model::task::TaskResult, prelude::*};

mod r#const;
use r#const as c;
pub use r#const::*;

#[path = "./enum.rs"]
mod sdlc_enum;
use sdlc_enum as e;
pub use sdlc_enum::{Status, *};

mod r#fn;
use r#fn as f;
use r#fn::*;

mod r#impl;
use r#impl as i;
use r#impl::*;

mod prompt;
use prompt as p;
use prompt::*;

mod r#struct;
use r#struct as s;
pub use r#struct::{AiSession, *};

#[path = "./trait.rs"]
pub mod sdlc_trait;
use sdlc_trait as t;
use sdlc_trait::*;

mod ui;
use ui as u;
pub use ui::*;

// cmd+alt+f
// - Search in all files overlay
// cmd+shift+f
// - Search in all files tab
// cmd+f
// - Search in file

// cargo nextest run -p estate -E 'test(/sdlc/)'
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn build_steps_are_small_and_sequential() {
		let steps = build_steps();
		assert!(!steps.is_empty());
		assert!(steps[0].contains("plan.md"));
		assert!(steps[1].contains("Inspect"));
		assert!(steps.iter().any(|s| s.contains("tests")));
		assert!(steps.iter().any(|s| s.contains("failures")));
		assert!(steps.last().unwrap().contains("remaining work"));
	}

	#[test]
	fn build_step_prompt_contains_step_and_context() {
		let prompt = build_step_prompt(
			"Read plan.md and identify the files that must be created or modified.",
			1,
			10,
			"Plan says to create hello-world.js.",
			"CWD: /project\nFILES:\nhello-world.js",
		);

		assert!(prompt.contains("BUILD STEP 1/10"));
		assert!(prompt.contains("hello-world.js"));
		assert!(prompt.contains("Plan says"));
	}
}
