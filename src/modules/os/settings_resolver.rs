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
// 3. VSCode settings workspace
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
	pub fn new_from_ref(target: impl AsRef<Path>) -> Self {
		let target = target.as_ref().to_path_buf();

		Self {
			root: Self::fs_root(&target),
			target,
		}
	}
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


#[cfg(test)]
mod tests {
	use super::*;
	use ::macros::vow;
	use serde_json::{Value, json};
	use std::{
		fs,
		path::{Path, PathBuf},
		sync::atomic::{AtomicU64, Ordering},
	};
	use crate::prelude::*;
	use crate::native::resolver::resolve_settings;

	const SETTINGS_FILENAME: &str = "settings.json";

	// Each file is on the target's ancestor path:
	//
	// root/settings.json                         default
	// root/future/settings.json                  profile
	// root/future/kb/project/settings.json       workspace
	// root/future/kb/project/crates/estate/
	//     settings.json                          project
	//
	// The workspace file is farther from the target than
	// the project file. It must still take precedence because
	// the JSON "type", not path proximity, determines priority.

	static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

	struct SettingsFixture {
		root: PathBuf,
		target: PathBuf,
	}

	impl SettingsFixture {
		fn new() -> std::io::Result<Self> {
			let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);

			let root = std::env::temp_dir().join(format!(
				"estate-settings-{}-{id}",
				std::process::id()
			));

			let target = root.join("future/kb/project/crates/estate");
			fs::create_dir_all(&target)?;

			let fixture = Self { root, target };

			fixture.write_default(json!({
				"type": "default",
				"log_path": false,
				"is_tray_visible": false
			}))?;

			fixture.write_profile(json!({
				"type": "profile",
				"log_path": true
			}))?;

			fixture.write_project(json!({
				"type": "project",
				"is_tray_app": true,
				"is_tray_visible": false
			}))?;

			fixture.write_workspace(json!({
				"type": "workspace",
				"is_tray_visible": true
			}))?;

			Ok(fixture)
		}
		fn default_path(&self) -> PathBuf {
			self.root.join(SETTINGS_FILENAME)
		}
		fn profile_path(&self) -> PathBuf {
			self.root.join("future").join(SETTINGS_FILENAME)
		}
		fn project_path(&self) -> PathBuf {
			self.target.join(SETTINGS_FILENAME)
		}
		fn workspace_path(&self) -> PathBuf {
			self.root
				.join("future/kb/project")
				.join(SETTINGS_FILENAME)
		}
		fn write(path: &Path, value: Value) -> std::io::Result<()> {
			fs::write(path, serde_json::to_string_pretty(&value)?)
		}
		fn write_default(&self, value: Value) -> std::io::Result<()> {
			Self::write(&self.default_path(), value)
		}
		fn write_profile(&self, value: Value) -> std::io::Result<()> {
			Self::write(&self.profile_path(), value)
		}
		fn write_project(&self, value: Value) -> std::io::Result<()> {
			Self::write(&self.project_path(), value)
		}
		fn write_workspace(&self, value: Value) -> std::io::Result<()> {
			Self::write(&self.workspace_path(), value)
		}

		fn resolve(&self) -> anyhow::Result<Value> {
			let settings =
				resolve_settings(&self.target, SETTINGS_FILENAME)?;
			Ok(serde_json::to_value(settings)?)
		}
	}

	impl Drop for SettingsFixture {
		fn drop(&mut self) {
			let _ = fs::remove_dir_all(&self.root);
		}
	}

	vow!(settings_resolver_finds_files, {
		let fixture = SettingsFixture::new().unwrap();
		let walker = FsWalker::new(&fixture.target);

		let found = walker.find_named(SETTINGS_FILENAME).unwrap();

		assert_eq!(found.len(), 4);
		assert!(found.contains(&fixture.default_path()));
		assert!(found.contains(&fixture.profile_path()));
		assert!(found.contains(&fixture.project_path()));
		assert!(found.contains(&fixture.workspace_path()));

		println!("Discovered settings:");
		for path in found {
			println!("  {}", path.display());
		}
	});

	vow!(settings_resolver_resolves_precedence, {
		let fixture = SettingsFixture::new().unwrap();
		let resolved = fixture.resolve().unwrap();

		// Workspace overrides project, even though project
		// is physically closer to the target.
		assert_eq!(resolved["type"], "workspace");
		assert_eq!(resolved["is_tray_visible"], true);

		// Values absent from workspace survive from lower layers.
		assert_eq!(resolved["is_tray_app"], true);
		assert_eq!(resolved["log_path"], true);

		println!(
			"Resolved settings:\n{}",
			serde_json::to_string_pretty(&resolved).unwrap()
		);
	});

	vow!(settings_resolver_falls_back, {
		let fixture = SettingsFixture::new().unwrap();

		fs::remove_file(fixture.workspace_path()).unwrap();

		let resolved = fixture.resolve().unwrap();

		assert_eq!(resolved["type"], "project");
		assert_eq!(resolved["is_tray_visible"], false);
		assert_eq!(resolved["is_tray_app"], true);
		assert_eq!(resolved["log_path"], true);
	});

	vow!(settings_resolver_falls_back_to_profile, {
		let fixture = SettingsFixture::new().unwrap();

		fs::remove_file(fixture.workspace_path()).unwrap();
		fs::remove_file(fixture.project_path()).unwrap();

		let resolved = fixture.resolve().unwrap();

		assert_eq!(resolved["type"], "profile");
		assert_eq!(resolved["log_path"], true);
		assert_eq!(resolved["is_tray_visible"], false);
	});

	vow!(settings_resolver_falls_back_to_default, {
		let fixture = SettingsFixture::new().unwrap();

		fs::remove_file(fixture.workspace_path()).unwrap();
		fs::remove_file(fixture.project_path()).unwrap();
		fs::remove_file(fixture.profile_path()).unwrap();

		let resolved = fixture.resolve().unwrap();

		assert_eq!(resolved["type"], "default");
		assert_eq!(resolved["log_path"], false);
		assert_eq!(resolved["is_tray_visible"], false);
	});

	vow!(settings_resolver_uses_json_type_not_filename, {
		let fixture = SettingsFixture::new().unwrap();

		// Deliberately swap the types without moving the files.
		fixture.write_project(json!({
			"type": "workspace",
			"is_tray_app": true,
			"is_tray_visible": true
		})).unwrap();

		fixture.write_workspace(json!({
			"type": "project",
			"is_tray_visible": false
		})).unwrap();

		let resolved = fixture.resolve().unwrap();

		assert_eq!(resolved["type"], "workspace");
		assert_eq!(resolved["is_tray_visible"], true);
	});

	vow!(settings_resolver_rejects_unknown_type, {
		let fixture = SettingsFixture::new().unwrap();

		fixture.write_workspace(json!({
			"type": "unrecognized",
			"is_tray_visible": true
		})).unwrap();

		assert!(fixture.resolve().is_err());
	});

	vow!(settings_resolver_rejects_missing_type, {
		let fixture = SettingsFixture::new().unwrap();

		fixture.write_workspace(json!({
			"is_tray_visible": true
		})).unwrap();

		assert!(fixture.resolve().is_err());
	});

	vow!(settings_resolver_rejects_invalid_json, {
		let fixture = SettingsFixture::new().unwrap();

		fs::write(
			fixture.workspace_path(),
			r#"{"type":"workspace","log_path":}"#,
		).unwrap();

		assert!(fixture.resolve().is_err());
	});

	vow!(settings_resolver_serializes_in_schema_order, {
		let fixture = SettingsFixture::new().unwrap();

		// Serialize the typed Settings directly.
		// Converting to Value first can reorder map keys.
		let settings =
			resolve_settings(&fixture.target, SETTINGS_FILENAME).unwrap();

		let output = serde_json::to_string_pretty(&settings).unwrap();

		let type_pos = output.find("\"type\"").unwrap();
		let tray_pos = output.find("\"is_tray_app\"").unwrap();
		let logging_pos = output.find("\"log_path\"").unwrap();

		// Adjust these assertions to your actual Settings
		// struct declaration order.
		assert!(type_pos < tray_pos);
		assert!(tray_pos < logging_pos);
	});
}