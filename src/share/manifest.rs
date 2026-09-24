use crate::prelude::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
	pub r#type: String,

	// Source metadata
	pub file_path: Option<PathBuf>,
	pub is_from: Option<String>,
	pub collision_key: Option<String>,

	// Application
	/// Do I want the menu bar tray to init an icon?
	pub has_tray_icon: Option<bool>,
	pub is_tray_app: Option<bool>,
	pub is_tray_visible: Option<bool>,
	pub log_path: Option<bool>,

	// Temporary test fields
	pub setting_from_type_project: Option<String>,
	pub setting_from_type_workspace: Option<String>,
}
