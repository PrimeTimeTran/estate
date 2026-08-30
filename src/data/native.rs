use std::sync::{Mutex, OnceLock, atomic::AtomicBool};

use core_graphics::display::{CGPoint, CGRect};
use egui::Color32;

use crate::{
	native::prelude::{ScrollRedirectState, *},
	ui::PanelConfig,
};

pub static INDEX_PATH: &str = ".config/estate/master.json";
pub static HOME_DIR: &str = ".config/estate";
pub const WORKSPACE_SETTINGS: &str = ".estate/settings.json";
pub static HMR_CHART_JSON: &str = "/Users/future/kb/project/crates/estate/src/data/chart.json";
pub const INTRINSIC_FILES: [&str; 3] = ["default.settings.json", "settings.json", "key-map.json"];

// pub static INITIAL_WINDOW: WindowType = WindowType::Dashboard;
// pub static INITIAL_WINDOW: WindowType = WindowType::TelemetryInspector;
// pub static INITIAL_WINDOW: WindowType = WindowType::TaskManager;
pub static INITIAL_WINDOW: WindowType = WindowType::WaterfallChart;
// pub static INITIAL_WINDOW: WindowType = WindowType::MarkdownView;

pub(crate) struct VeConfig {
	pub bg: Color32,
	pub surface: Color32,
	pub activity_bar: PanelConfig,
	pub primary_bar: PanelConfig,
	pub secondary_bar: PanelConfig,
	pub bottom_panel: PanelConfig,
	pub status_bar: PanelConfig,
	pub dock_left: PanelConfig,
	pub dock_right: PanelConfig,
}

pub(crate) const DEFAULT_CONFIG: VeConfig = VeConfig {
	bg: palette::BG,
	surface: palette::SURFACE,
	activity_bar: PanelConfig::new(true, 48.0),
	primary_bar: PanelConfig::new(true, 40.0),
	secondary_bar: PanelConfig::new(true, 48.0),
	bottom_panel: PanelConfig::new(true, 0.0),
	status_bar: PanelConfig::new(true, 24.0),
	dock_left: PanelConfig::new(true, 280.0),
	dock_right: PanelConfig::new(true, 320.0),
};

pub fn master() -> serde_json::Value {
	serde_json::json!({
		"cache": {
			"hotkey.configs": [],
			"indexes": []
		},
		"logs": {
			"events": [],
			"sessions": [],
			"hotkey.triggers": [],
			"notifications": []
		},
		"session": {
			"start": null,
			"end": null,
			"events": [],
			"hotkey.triggers": [],
			"file.participants": [],
			"index.current": {
				"start": null,
				"end": null,
				"active": false,
				"status": null
			}
		},
		"metrics": {
			"anchors": 0,
			"errors": 0,
			"events": 0,
			"files": 0,
			"hotkeys": 0,
			"types": 0,
			"notifications": 0,
			"files.unique": 0,
			"counter": [],
			"wikilinks": {
				"tags": 0,
				"unique": 0,
				"active": 0,
				"unresolved": 0,
				"aliases": 0
			},
			"projects": {
				"npm": 0,
				"cargo": 0,
				"estate": 0
			},
			"index": {
				"active": false,
				"time.max": 0,
				"time.min": 0,
				"last.status": null,
				"last.run.datetime": null,
				"last.run.time": null
			},
			"du": {
				"src": "0b",
				"deps": "0b",
				"unwatched": "0b",
				"log.files": "0b"
			},
			"hotkey.triggers": {
				"f13": 0,
				"f24": 0,
				"cmd": 0,
				"ctrl": 0,
				"shift": 0,
				"space": 0,
				"chords": 0,
				"hyperkeys": 0,
				"double.taps": 0,
				"leaderkeys": 0
			}
		},
		"config.active": {
			"sources": [],
			"version.go": null,
			"version.python": null,
			"version.rust": null,
			"workspace.cwd": null,
			"estate.items.index.dir": "",
			"create.estate.resource.write.dir": "index-dir/cwd/sibling-of-focused-file",
			"create.estate.wikilink.file.name.pattern": "match-unresolved|unix-safe",
			"estate.files.exclude": [],
			"estate.search.include": [],
			"ai": {
				"active": false
			}
		},
		"estate": {
			"types": "",
			"orphans": [],
			"files": [],
			"tags": [],
			"pipelines": [],
			"onboarding": {
				"has.remote": false,
				"has.dotrepo": false,
				"has.profile": false,
				"keymaps": {
					"has.f13": false,
					"has.f24": false,
					"has.chord": false,
					"has.hyperkey": false,
					"has.double.tap": false,
					"has.leaderkey": false
				}
			}
		},
		"resources": [],
		"inodes": []
	})
}
