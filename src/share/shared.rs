use crate::prelude::*;

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ArtifactKind {
	#[default]
	File,
	Markdown,
	Mermaid,
	Image,
	CodeExample,
	TestCase,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TextRange {
	pub start: TextPosition,
	pub end: TextPosition,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TextPosition {
	pub line: u32,
	pub column: u32,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Visibility {
	#[default]
	Public,
	Personal,
	Private,
	Team,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Scope {
	pub owner: Option<String>,
	pub visibility: Visibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Resource {
	pub id: Uuid,
	pub kind: ResourceKind,
	pub locations: Vec<ResourceLocation>,
	pub aliases: Vec<Alias>,
	pub meta: ResourceMetadata, // apparently this is the actual field
}
impl Default for Resource {
	fn default() -> Self {
		Self {
			id: Uuid::new_v4(),
			kind: ResourceKind::File,
			locations: vec![],
			aliases: vec![],
			meta: ResourceMetadata::default(),
		}
	}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
	File,
	Symbol,
	Anchor,
	Workspace,
	Project,
	Generated,
	Config,
	Source,
	Document,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ResourceMetadata {
	pub created_at: u64,
	pub updated_at: u64,
	pub scope: Scope,
}
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResourceLocation {
	File(PathBuf),
	Directory(PathBuf),
	Url(String),
	External { provider: String, id: String },
}
impl Default for ResourceLocation {
	fn default() -> Self {
		Self::File(PathBuf::new())
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Alias {
	pub name: String,
	pub resource: Uuid,
}
//######"What bytes exist?"
// #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub struct Resource {
// 	pub id: ResourceId,
// 	pub kind: ResourceKind,
// 	pub location: ResourceLocation,
// 	pub range: Option<TextRange>,
// 	pub git: Option<GitAnchor>,
// }
// impl Resource {
// 	pub fn from_path(path: PathBuf) -> Self {
// 		Self {
// 			id: ResourceId(0),
// 			kind: ResourceKind::from_path(&path),
// 			location: ResourceLocation::File(path),
// 			range: None,
// 			git: None,
// 		}
// 	}
// }
// #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub enum ResourceKind {
// 	#[default]
// 	Source,
// 	Document,
// 	Image,
// 	Diagram,
// 	Test,
// 	Config,
// 	Generated,
// 	Directory,
// }
// impl ResourceKind {
// 	pub fn from_path(path: &Path) -> Self {
// 		if path.is_dir() {
// 			return ResourceKind::Directory;
// 		}
// 		match path.extension().and_then(|e| e.to_str()) {
// 			Some("rs") => ResourceKind::Source,
// 			Some("md") => ResourceKind::Document,
// 			Some("png" | "jpg" | "jpeg" | "svg") => ResourceKind::Image,
// 			Some("mmd" | "mermaid") => ResourceKind::Diagram,
// 			Some("json" | "toml" | "yaml" | "yml") => ResourceKind::Config,
// 			_ => ResourceKind::Generated,
// 		}
// 	}
// }
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Tag {
	pub id: Uuid,
	pub name: String,
	pub caption: String,
	pub body: String,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TagBinding {
	pub tag: Uuid,
	pub target: Uuid,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Binding {
	pub node: Uuid,
	pub resource: Uuid,
	pub range: Option<TextRange>,
	pub git: Option<GitAnchor>,
}
impl Binding {
	pub fn new(node: Uuid, resource: Uuid) -> Self {
		Self {
			node,
			resource,
			range: None,
			git: None,
		}
	}
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Bookmark {
	pub id: Uuid,
	// What am I returning to?
	pub resource: Uuid,
	// Optional organization
	pub tags: Vec<Tag>,
	// Optional semantic grouping
	pub concepts: Vec<Uuid>,
	pub note: Option<String>,
	pub scope: Scope,
	pub created_at: DateTime<Utc>,
}
impl Bookmark {
	pub fn new(resource: Uuid, scope: Scope) -> Self {
		Self {
			id: Uuid::new_v4(),
			resource,
			tags: Vec::new(),
			concepts: Vec::new(),
			note: None,
			scope,
			created_at: Utc::now(),
		}
	}
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct GitAnchor {
	pub repository: String,
	pub commit: String,
	pub branch: Option<String>,
	pub hash: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SectionConfig {
	pub title: Option<String>,
	pub kind: Option<String>,
	pub path: Option<String>,
	pub description: Option<String>,
	pub metadata: serde_yaml::Value,
	pub relations: serde_yaml::Value,
}
impl Default for SectionConfig {
	fn default() -> Self {
		Self {
			title: None,
			kind: None,
			path: Some("./".to_string()),
			description: None,
			metadata: serde_yaml::Value::Null,
			relations: serde_yaml::Value::Null,
		}
	}
}
#[derive(Debug)]
pub struct EstateDocument {
	pub frontmatter: SectionConfig,
	pub sections: Vec<RawSection>,
}
#[derive(Debug)]
pub struct RawSection {
	pub heading: String,
	pub frontmatter: SectionConfig,
	pub items: Vec<String>,
}
#[derive(Debug)]
pub struct EstateSection {
	pub config: SectionConfig,
	pub items: Vec<String>,
}

// pub type ResourceId = u64;
// pub struct Resource {
//     pub id: ResourceId,
//     pub kind: ResourceKind,
// }
// pub enum ResourceLocation {
//     File {
//         path: PathBuf,
//         inode: Option<u64>,
//     },
//     Git {
//         repo: String,
//         commit: String,
//         path: String,
//     },
//     Remote {
//         uri: String,
//     },
// }

// pub struct Alias {
//     pub name: String,
//     pub resource: ResourceId,
// }

// pub struct Anchor {
//     pub name: String,
//     pub resource: ResourceId,
// }

// pub struct Edge {
//     pub from: ResourceId,
//     pub to: ResourceId,
//     pub kind: EdgeKind,
// }

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Viewed {
	pub id: Uuid,
	pub name: String,
	pub filters: Vec<ViewFilter>,
	pub layout: Layout,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ViewFilter {
	#[default]
	Developer,
	NodeKind(NodeKind),
	ResourceKind(ResourceKind),
	Tag(String),
	Scope(Visibility),
	Relation(RelationKind),
	NameContains(String),
	CreatedAfter(DateTime<Utc>),
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Layout {
	#[default]
	List,
	Tree,
	Graph,
	Canvas,
	Table,
	Timeline,
	Kanban,
}
