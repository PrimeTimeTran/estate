// To Build an app like Ive been talking about robustly
// I've realized I need to handle the same tiered pattern across many apps
//
// For example settings can be default, global, or workspace.
// And also the schema that is bound to it.
//
// estate.settings.schema.json
// estate.settings.0-default.json
// estate.settings.1-global.json
// estate.settings.2-project.json
// estate.settings.3-workspace.json
//
// I need to build a small module which does this, as a resolver.
// and I want it to be generic, and passed in a 'base name' where it finds that name
// at the different 'layer'.
//
// For example
// 1. VSCode settings global
// 2. VSCode settings project
// 3. VScode settings workspace
//
// They all have different page but the same shape and name.
// So I want to do this for my estate project again.

use crate::prelude::*;

#[derive(Debug)]
pub struct FsWalker {
	root: PathBuf,
	target: PathBuf,
}

impl FsWalker {
	pub fn new(target: impl Into<PathBuf>) -> Self {
		let target = target.into();

		Self {
			root: Self::fs_root(&target),
			target,
		}
	}
	fn fs_root(path: &Path) -> PathBuf {
		path.ancestors().last().unwrap().to_path_buf()
	}
	pub fn find_named(&self, base_name: &str) -> std::io::Result<Vec<PathBuf>> {
		let mut found = Vec::new();
		self.walk_up_to_target(|dir| {
			let path = dir.join(base_name);
			if path.is_file() {
				found.push(path);
			}
		})?;

		Ok(found)
	}

	pub fn walk_up_to_target<F>(&self, mut visit: F) -> std::io::Result<()>
	where
		F: FnMut(&Path),
	{
		let mut current = self.root.clone();
		loop {
			visit(&current);
			if current == self.target {
				break;
			}
			let next = current.join(
				self
					.target
					.strip_prefix(&current)
					.unwrap()
					.components()
					.next()
					.unwrap(),
			);
			current = next;
		}
		Ok(())
	}
}

// Example resolution hierarchy:
//
// /Users/install-of-app-bin-tool-framework/settings.default.json
// /Users/future/personal/settings.global.json
// /Users/future/kb/project/settings.workspace.json
// /Users/future/kb/project/crates/estate/settings.project.json
//

// cargo nextest run -p estate -E 'test(/settings_resolver_/)'
// cargo nextest run -p estate -E 'test(settings_resolver_finds_files) or test(settings_resolver_resolves_precedence) or test(settings_resolver_falls_back)'
const SETTINGS_FILES: &[&str] = &[
	// Lowest precedence
	"settings.default.json",
	// Global personal
	"settings.global.json",
	// Project specific
	"settings.project.json",
	// Highest precedence
	"settings.workspace.json",
];
fn find_settings(walker: &FsWalker) -> std::io::Result<Vec<PathBuf>> {
	let mut found = Vec::new();
	for name in SETTINGS_FILES {
		found.extend(walker.find_named(name)?);
	}
	Ok(found)
}
fn create_settings_fixture() -> std::io::Result<(PathBuf, PathBuf)> {
	let root = std::env::temp_dir().join(format!("estate-fs-walker-{}", std::process::id()));
	let project = root.join("future/kb/project");
	let target = project.join("crates/estate");
	std::fs::create_dir_all(root.join("install-of-app-bin-tool-framework"))?;
	std::fs::create_dir_all(root.join("future/personal"))?;
	std::fs::create_dir_all(&target)?;
	std::fs::write(
		root.join("install-of-app-bin-tool-framework/settings.default.json"),
		r#"{"source":"default"}"#,
	)?;
	std::fs::write(
		root.join("future/personal/settings.global.json"),
		r#"{"source":"global"}"#,
	)?;
	std::fs::write(
		project.join("settings.workspace.json"),
		r#"{"source":"workspace"}"#,
	)?;
	std::fs::write(
		target.join("settings.project.json"),
		r#"{"source":"project"}"#,
	)?;
	Ok((root, target))
}

#[test]
fn settings_resolver_finds_files() -> std::io::Result<()> {
	let (root, target) = create_settings_fixture()?;
	let walker = FsWalker::new(&target);
	let found = find_settings(&walker)?;
	println!("\nSettings files found:");
	for path in &found {
		println!("  {}", path.display());
	}
	assert_eq!(found.len(), 2);
	assert!(found.iter().any(|p| p.ends_with("settings.workspace.json")));
	assert!(found.iter().any(|p| p.ends_with("settings.project.json")));
	std::fs::remove_dir_all(root)?;
	Ok(())
}


#[test]
fn settings_resolver_resolves_precedence() -> std::io::Result<()> {
	let (root, target) = create_settings_fixture()?;
	let walker = FsWalker::new(&target);
	let names = [
		"settings.default.json",
		"settings.global.json",
		"settings.workspace.json",
		"settings.project.json",
	];

	let mut found = Vec::new();
	for name in names {
		found.extend(walker.find_named(name)?);
	}
	// For now, make the precedence explicit:
	// workspace > project > global > default
	let resolved = found
		.iter()
		.find(|path| path.ends_with("settings.project.json"))
		.unwrap();
	let contents = std::fs::read_to_string(resolved)?;
	assert!(contents.contains(r#""source":"project""#));
	println!("\nResolved settings:");
	println!("  {}", resolved.display());
	println!("  {}", contents);
	std::fs::remove_dir_all(root)?;
	Ok(())
}

#[test]
fn settings_resolver_falls_back() -> std::io::Result<()> {
	let (root, target) = create_settings_fixture()?;
	std::fs::remove_file(target.join("settings.project.json"))?;
	let walker = FsWalker::new(&target);
	let names = [
		"settings.default.json",
		"settings.global.json",
		"settings.workspace.json",
		"settings.project.json",
	];
	let mut found = Vec::new();
	for name in names {
		found.extend(walker.find_named(name)?);
	}
	let resolved = found
		.iter()
		.find(|path| path.ends_with("settings.workspace.json"))
		.unwrap();
	let contents = std::fs::read_to_string(resolved)?;
	assert!(contents.contains(r#""source":"workspace""#));
	println!("\nResolved settings after removing project:");
	println!("  {}", resolved.display());
	println!("  {}", contents);
	std::fs::remove_dir_all(root)?;
	Ok(())
}
