pub use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout as RatatuiLayout, Position, Rect},
	widgets::Clear,
};

use anyhow::{Context, anyhow};
use crossterm::{
	cursor,
	event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use jev_sdk::{Choice, Noul, Question, Score, TypeSafeClient};
use std::path::{Path, PathBuf};
use std::{io::Stdout, process::Command};
use tokio::time::{Duration, sleep};

use crate::{model::task::TaskResult, prelude::*};

mod r#const;
use r#const as c;
use r#const::*;

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

#[derive(Debug, Clone)]
pub struct RequiredFile {
	pub name: &'static str,
	pub min_chars: usize,
}
#[derive(Debug, Clone)]
pub struct FileCheck {
	pub path: PathBuf,
	pub exists: bool,
	pub chars: usize,
	pub meets_requirement: bool,
}
impl RequiredFile {
	pub fn check(&self, dir: impl AsRef<Path>) -> FileCheck {
		let path = dir.as_ref().join(self.name);

		match std::fs::read_to_string(&path) {
			Ok(content) => {
				let chars = content.chars().count();

				FileCheck {
					path,
					exists: true,
					chars,
					meets_requirement: chars >= self.min_chars,
				}
			}

			Err(_) => FileCheck {
				path,
				exists: false,
				chars: 0,
				meets_requirement: false,
			},
		}
	}
}
pub fn check_required_files(dir: impl AsRef<Path>, files: &[RequiredFile]) -> Vec<FileCheck> {
	files.iter().map(|file| file.check(&dir)).collect()
}
fn checkall() {
	// let checks = check_required_files(session_dir, &required_files);
	// for check in &checks {
	// 	println!(
	// 		"{:<12} exists={} chars={} valid={}",
	// 		check.path.file_name().unwrap().to_string_lossy(),
	// 		check.exists,
	// 		check.chars,
	// 		check.meets_requirement,
	// 	);
	// }

	// 	let checks = check_required_files(session_dir, &required_files);
	// 	if all_requirements_met(&checks) {
	// 		// deterministic behavior
	// 	}
	//
	// 	let context = inspect_workspace(session_dir, &required_files);
	//
	// 	if context.all_ready() {
	// 		// choose the next deterministic action
	// 	}
	//
	// 	fn select_prompt(context: &DisclosureContext) -> &'static str {
	// 		if context.all_ready() {
	// 			BUILD_PROMPT
	// 		} else if context.files.iter().any(|f| f.path.ends_with("plan.md")) {
	// 			PLAN_PROMPT
	// 		} else if context.files.iter().any(|f| f.path.ends_with("spec.md")) {
	// 			SPEC_PROMPT
	// 		} else {
	// 			INTENT_PROMPT
	// 		}
	// 	}
	let required_files = vec![
		RequiredFile {
			name: "intent.md",
			min_chars: 100,
		},
		RequiredFile {
			name: "spec.md",
			min_chars: 100,
		},
		RequiredFile {
			name: "plan.md",
			min_chars: 100,
		},
	];
	pub fn all_requirements_met(checks: &[FileCheck]) -> bool {
		checks.iter().all(|check| check.meets_requirement)
	}
	let rules = vec![
		DiscoveryRule {
			name: "rust",
			paths: &["Cargo.toml", "Cargo.lock"],
			min_chars: Some(20),
			disclosure: Disclosure::ProjectType("Rust"),
		},
		DiscoveryRule {
			name: "sdlc-intent",
			paths: &["intent.md"],
			min_chars: Some(100),
			disclosure: Disclosure::Context("SDLC intent"),
		},
		DiscoveryRule {
			name: "sdlc-spec",
			paths: &["spec.md"],
			min_chars: Some(100),
			disclosure: Disclosure::Context("SDLC specification"),
		},
		DiscoveryRule {
			name: "sdlc-plan",
			paths: &["plan.md"],
			min_chars: Some(100),
			disclosure: Disclosure::Context("SDLC plan"),
		},
	];
	// let workspace = CtxWorkspace::default();
	let workspace = Path::new(".");
	let evidence = discover(workspace, &rules);
	for item in &evidence {
		println!("{item:#?}");
	}
}
#[derive(Debug)]
pub struct DiscoveryContext {
	pub evidence: Vec<Evidence>,
}
impl DiscoveryContext {
	pub fn matched(&self, name: &str) -> bool {
		self.evidence.iter().any(|e| e.rule == name && e.matched)
	}
}
#[derive(Debug, Clone)]
pub struct DiscoveryRule {
	pub name: &'static str,
	pub paths: &'static [&'static str],
	pub min_chars: Option<usize>,
	pub disclosure: Disclosure,
}
#[derive(Debug, Clone)]
pub enum Disclosure {
	ProjectType(&'static str),
	Context(&'static str),
}
#[derive(Debug, Clone)]
pub struct Evidence {
	pub rule: &'static str,
	pub paths: Vec<PathEvidence>,
	pub matched: bool,
}
#[derive(Debug, Clone)]
pub struct PathEvidence {
	pub path: PathBuf,
	pub exists: bool,
	pub chars: Option<usize>,
}

impl DiscoveryRule {
	pub fn inspect(&self, root: &Path) -> Evidence {
		let paths = self
			.paths
			.iter()
			.map(|relative| {
				let path = root.join(relative);

				let chars = std::fs::read_to_string(&path)
					.ok()
					.map(|content| content.chars().count());

				PathEvidence {
					path,
					exists: chars.is_some(),
					chars,
				}
			})
			.collect::<Vec<_>>();

		let matched = paths.iter().any(|evidence| {
			evidence.exists
				&& self
					.min_chars
					.map_or(true, |min| evidence.chars.unwrap_or(0) >= min)
		});

		Evidence {
			rule: self.name,
			paths,
			matched,
		}
	}
}
trait Discovery {
	fn inspect(&self, root: &Path) -> DiscoveryResult;
}
struct DiscoveryResult;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
	Rust,
	Python,
	Go,
	Node,
	Unknown,
}
pub fn discover_project_type(root: &Path) -> ProjectType {
	let cargo = root.join("Cargo.toml");
	let go = root.join("go.mod");
	let python = root.join("pyproject.toml");
	let package = root.join("package.json");

	if cargo.is_file() {
		ProjectType::Rust
	} else if go.is_file() {
		ProjectType::Go
	} else if python.is_file() {
		ProjectType::Python
	} else if package.is_file() {
		ProjectType::Node
	} else {
		ProjectType::Unknown
	}
}
pub fn discover(root: impl AsRef<Path>, rules: &[DiscoveryRule]) -> Vec<Evidence> {
	rules
		.iter()
		.map(|rule| rule.inspect(root.as_ref()))
		.collect()
}

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
