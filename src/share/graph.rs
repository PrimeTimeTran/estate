use crate::prelude::*;

/// A semantic entity in the Estate graph.
///
/// A [`Node`] represents a named thing independently of where that thing is
/// stored or how it is represented. A node may correspond to a source file,
/// code snippet, concept, document, workspace, artifact, or other semantic
/// entity.
///
/// This distinction allows multiple resources to refer to the same node. For
/// example, several implementations, examples, or discussions of the same
/// idea may all be associated with one node:
///
/// ```text
/// Node: "climbingStairs"
/// ├── python implementation
/// ├── rust implementation
/// ├── blog post explaining the algorithm
/// └── notes comparing recursive and dynamic-programming solutions
/// ```
///
/// Nodes form the semantic layer of an Estate. [`Relation`]s connect nodes
/// to describe how those entities relate to one another.
///
/// # Semantic Relationships
///
/// Relations can express relationships that exist independently of the
/// physical organization of files. For example:
///
/// ```text
/// VFS
/// ├── Implements ──→ Inode-based filesystem
/// ├── ExplainedBy ──→ compiler architecture article
/// ├── UsedBy ──────→ WASM IDE
/// └── RelatedTo ───→ client-side sandbox
/// ```
///
/// A source file or code block therefore does not necessarily *define* the
/// semantic entity it contains. Instead, it can provide a representation of,
/// implementation of, explanation of, or reference to a node.
///
/// # Projections
///
/// Nodes are also intended to serve as the basis for projections over the
/// semantic graph. A projection may present the same underlying nodes
/// according to a particular concern, such as:
///
/// - all implementations of a concept;
/// - all resources associated with a node;
/// - all code related to a particular API;
/// - all explanations of a concept;
/// - all nodes reachable through a particular relationship;
/// - a filesystem-oriented view of an otherwise semantic graph.
///
/// The node therefore represents **what something is**, while resources,
/// bindings, relations, and projections describe **where it is, how it is
/// represented, and how it relates to other things**.
///
/// # Identity
///
/// The [`Uuid`] identifies the semantic entity rather than a particular
/// resource containing it. This permits multiple resources or representations
/// to refer to the same node without duplicating its semantic identity.
///
/// # Examples
///
/// A node representing a conceptual implementation can exist independently
/// of the files containing its implementations:
///
/// ```
/// # use estate::prelude::*;
///
/// let climbing_stairs = Node::new(
///     "climbingStairs".into(),
///     NodeKind::Concept,
///     Some("Ways of solving the climbing stairs problem.".into()),
///     Scope::default(),
///     None,
/// );
/// ```
///
/// Individual implementations or explanations can then be represented by
/// other nodes and connected with [`Relation`]s such as [`RelationKind::Implements`],
/// [`RelationKind::Explains`], or [`RelationKind::References`].
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Node {
	/// The stable identity of this semantic entity.
	pub id: Uuid,

	/// The semantic category of the node.
	pub kind: NodeKind,

	/// The human-readable name of the entity.
	pub name: String,

	/// An optional description of what the entity represents.
	pub description: Option<String>,

	/// The scope in which this entity is meaningful.
	pub scope: Scope,

	/// Optional semantic tags associated with the entity.
	pub tags: Option<Vec<Tag>>,

	/// When this node was created.
	pub created_at: DateTime<Utc>,

	/// When this node was last modified.
	pub updated_at: DateTime<Utc>,
}

// impl Node {
// 	/// Creates a new semantic node.
// 	///
// 	/// The node receives a new time-sortable [`Uuid`] and uses the current
// 	/// time for both [`Node::created_at`] and [`Node::updated_at`].
// 	pub fn new(
// 		name: String,
// 		kind: NodeKind,
// 		description: Option<String>,
// 		scope: Scope,
// 		tags: Option<Vec<Tag>>,
// 	) -> Self {
// 		let now = Utc::now();

// 		Self {
// 			id: Uuid::new_v4(),
// 			kind,
// 			name,
// 			description,
// 			scope,
// 			tags,
// 			created_at: now,
// 			updated_at: now,
// 		}
// 	}
// }

// /// Graph
// ///     "What is A connected to?"
// /// - I created an estate .md file which wikilinks to 5 other estate files. Do I do a full table scan of the registry every time? No, the resolver should take in an estate id and context and give me back what it is I'm looking for. If I've opened the IDE from a repo/workspace then the link will look differently to resolve.
// pub trait Graph {
// 	fn children(&self, id: Uuid) -> Vec<Uuid>;
// 	fn parents(&self, id: Uuid) -> Vec<Uuid>;
// 	fn dependencies(&self, id: Uuid) -> Vec<Uuid>;
// }

// // #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// // pub enum NodeKind {
// // 	#[default]
// // 	Generic,
// // 	Concept,
// // 	Workspace,
// // 	Bookmark,
// // 	Artifact,
// // 	View,
// // 	Asset,
// // 	Config,
// // 	Directory,
// // 	Document,
// // 	File,
// // 	Source,
// // }

// // // "What named thing exists?"
// // #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// // pub struct Relation {
// // 	pub from: Uuid,
// // 	pub to: Uuid,
// // 	pub kind: RelationKind,
// // }
// // impl Relation {
// // 	pub fn new(from: Uuid, to: Uuid, kind: RelationKind) -> Self {
// // 		Self { from, to, kind }
// // 	}
// // }
// // #[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
// // pub enum RelationKind {
// // 	#[default]
// // 	Null,
// // 	Represents,
// // 	Contains,
// // 	PartOf,
// // 	DependsOn,
// // 	Implements,
// // 	Uses,
// // 	Documents,
// // 	Explains,
// // 	Visualizes,
// // 	DerivedFrom,
// // 	GeneratedFrom,
// // 	RelatedTo,
// // 	References,
// // 	Imports,
// // 	Calls,
// // }

/// ---------------------

/// Describes the semantic meaning of a [`Relation`].
///
/// Relation kinds allow the Estate graph to represent semantic connections
/// between entities without requiring those entities to share a filesystem
/// location or representation.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum RelationKind {
	/// A relationship with no more specific semantic meaning.
	#[default]
	Null,

	/// The source entity represents the target entity.
	Represents,

	/// The source entity contains the target entity.
	Contains,

	/// The source entity is a part of the target entity.
	PartOf,

	/// The source entity depends on the target entity.
	DependsOn,

	/// The source entity provides an implementation of the target entity.
	Implements,

	/// The source entity uses the target entity.
	Uses,

	/// The source entity documents the target entity.
	Documents,

	/// The source entity explains the target entity.
	Explains,

	/// The source entity provides a visualization of the target entity.
	Visualizes,

	/// The source entity was derived from the target entity.
	DerivedFrom,

	/// The source entity was generated from the target entity.
	GeneratedFrom,

	/// The source entity is generally related to the target entity.
	RelatedTo,

	/// The source entity references the target entity.
	References,

	/// The source entity imports the target entity.
	Imports,

	/// The source entity invokes or calls the target entity.
	Calls,
}

/// The semantic category of a [`Node`].
///
/// `NodeKind` describes what a node *represents*, rather than where it is
/// stored. A single semantic concept may therefore have many resources or
/// representations associated with it.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum NodeKind {
	/// A node without a more specific semantic classification.
	#[default]
	Generic,

	/// An abstract idea or concept.
	Concept,

	/// A workspace or project-level entity.
	Workspace,

	/// A saved point of interest.
	Bookmark,

	/// A produced or constructed artifact.
	Artifact,

	/// A derived presentation or view of other entities.
	View,

	/// An external or supporting asset.
	Asset,

	/// Configuration or configuration-related data.
	Config,

	/// A directory or hierarchical container.
	Directory,

	/// A document containing information or explanation.
	Document,

	/// A file-level semantic entity.
	File,

	/// A source-code entity or source representation.
	Source,
}

// "What named thing exists?"

/// A directed semantic relationship between two [`Node`]s.
///
/// A relation describes how one semantic entity relates to another. Relations
/// are independent of the physical location of either node, allowing the
/// semantic graph to connect entities across files, documents, projects, and
/// other resources.
///
/// The `from` node is the subject of the relationship and `to` is its target.
///
/// # Examples
///
/// A concrete implementation can be related to a conceptual entity:
///
/// ```text
/// Rust VFS implementation
///        │
///        └── Implements ──→ Virtual File System
/// ```
///
/// Multiple implementations can therefore point to the same semantic node:
///
/// ```text
/// Rust implementation ──┐
/// Python implementation ├── Implements ──→ Climbing Stairs
/// Blog explanation ─────┘
/// ```
///
/// The graph can consequently represent relationships that cannot be
/// expressed naturally by a filesystem hierarchy.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Relation {
	/// The source node of the relationship.
	pub from: Uuid,

	/// The target node of the relationship.
	pub to: Uuid,

	/// The semantic meaning of the relationship.
	pub kind: RelationKind,
}

impl Relation {
	/// Creates a directed semantic relationship between two nodes.
	pub fn new(from: Uuid, to: Uuid, kind: RelationKind) -> Self {
		Self { from, to, kind }
	}
}
