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
	pub id: Uuid,
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
			id: Uuid::now_v7(),
			kind,
			name,
			description,
			scope,
			tags,
			created_at: now,
			updated_at: now,
		}
	}
	// pub fn path<'a>(
	// 	&self,
	// 	resources: &'a [Resource],
	// 	bindings: &'a [Binding],
	// ) -> Option<&'a PathBuf> {
	// 	let binding = bindings.iter().find(|b| b.node == self.uid)?;
	// 	let resource = resources.iter().find(|r| r.id == binding.resource)?;
	// 	match &resource.location {
	// 		ResourceLocation::File(path) => Some(path),
	// 		_ => None,
	// 	}
	// }
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
	References,
	Imports,
	Calls,
}
