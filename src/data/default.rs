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
