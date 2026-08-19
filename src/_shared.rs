use crate::prelude::*;
//------------------------------------------------------------------------------------
// Node
// A node in a semantic graph representation.
// In a compiler, what is a parser? Many things. Answer concept, step, series, orchestrator, file and
// you wouldn't be wrong.
// kind:
// description:
//------------------------------------------------------------------------------------
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Node {
	pub uid: Uuid,
	pub kind: NodeKind,
	pub name: String,
	pub description: Option<String>,
	pub scope: Scope,
	pub tags: Option<Vec<Tag>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}
impl Node {
	pub fn new(
		name: String,
		kind: NodeKind,
		description: Option<String>,
		scope: Scope,
		tags: Option<Vec<Tag>>,
	) -> Self {
		let now = Utc::now();
		Self {
			uid: Uuid::now_v7(),
			kind,
			name,
			description,
			scope,
			tags,
			created_at: now,
			updated_at: now,
		}
	}
	pub fn path<'a>(
		&self,
		resources: &'a [Resource],
		bindings: &'a [Binding],
	) -> Option<&'a PathBuf> {
		let binding = bindings.iter().find(|b| b.node == self.uid)?;
		let resource = resources.iter().find(|r| r.uid == binding.resource)?;
		match &resource.location {
			ResourceLocation::File(path) => Some(path),
			_ => None,
		}
	}
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum NodeKind {
	#[default]
	Generic,
	Concept,
	Workspace,
	Bookmark,
	Artifact,
	View,
	Asset,
	Config,
	Directory,
	Document,
	File,
	Source,
}
// "What named thing exists?"
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Relation {
	pub from: Uuid,
	pub to: Uuid,
	pub kind: RelationKind,
}
impl Relation {
	pub fn new(from: Uuid, to: Uuid, kind: RelationKind) -> Self {
		Self { from, to, kind }
	}
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum RelationKind {
	#[default]
	Null,
	Represents,
	Contains,
	PartOf,
	DependsOn,
	Implements,
	Uses,
	Documents,
	Explains,
	Visualizes,
	DerivedFrom,
	GeneratedFrom,
	RelatedTo,
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct View {
	pub uid: Uuid,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Estate {
	pub uid: Uuid,
	pub name: String,
	pub parent: Option<Uuid>,
	pub scope: Scope,
	pub nodes: Vec<Node>,
	// pub nodes: Vec<Uuid>,
	pub resources: Vec<Resource>,
	pub relations: Vec<Relation>,
	pub bindings: Vec<Binding>,
}
impl Estate {
	pub fn new(name: String, scope: Scope) -> Self {
		Self {
			uid: Uuid::now_v7(),
			name,
			parent: None,
			scope,
			nodes: Vec::new(),
			resources: Vec::new(),
			relations: Vec::new(),
			bindings: Vec::new(),
		}
	}
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Artifact {
	pub uid: Uuid,
	pub kind: ArtifactKind,
	pub resource: Uuid,
}
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
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
//######"What bytes exist?"
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Resource {
	pub uid: Uuid,
	pub kind: ResourceKind,
	pub location: ResourceLocation,
	pub range: Option<TextRange>,
	pub git: Option<GitAnchor>,
}
impl Resource {
	pub fn from_path(path: PathBuf) -> Self {
		Self {
			uid: Uuid::now_v7(),
			kind: ResourceKind::from_path(&path),
			location: ResourceLocation::File(path),
			range: None,
			git: None,
		}
	}
}
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResourceKind {
	#[default]
	Source,
	Document,
	Image,
	Diagram,
	Test,
	Config,
	Generated,
	Directory,
}
impl ResourceKind {
	pub fn from_path(path: &Path) -> Self {
		if path.is_dir() {
			return ResourceKind::Directory;
		}
		match path.extension().and_then(|e| e.to_str()) {
			Some("rs") => ResourceKind::Source,
			Some("md") => ResourceKind::Document,
			Some("png" | "jpg" | "jpeg" | "svg") => ResourceKind::Image,
			Some("mmd" | "mermaid") => ResourceKind::Diagram,
			Some("json" | "toml" | "yaml" | "yml") => ResourceKind::Config,
			_ => ResourceKind::Generated,
		}
	}
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
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Tag {
	pub uid: Uuid,
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
	pub uid: Uuid,
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
			uid: Uuid::now_v7(),
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
