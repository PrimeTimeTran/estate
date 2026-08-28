use crate::{ prelude::* };

use crate::app::{ Runtime };

mod estate;
use estate::Estate;

#[derive(Clone, Debug)]
pub(crate) struct EstateEngine<R: Runtime> {
	// Domain
	pub estate: Estate,
	pub runtime: Arc<R>,

	// Infrastructure
	pub vfs: EstateVfs,
	pub index: EstateIndex,
	pub workspace: Workspace,

	// pub index: OnceCell<EstateIndex>,
	// pub search: OnceCell<SearchService>,
	// pub analysis: OnceCell<AnalysisService>,

	// domain representation
	pub graph: EstateGraph,

	// persistence/coordination
	pub registry: EstateRegistry,

	/// Capabilities
	pub resolver: EstateResolver,
	pub discovery: EstateDiscovery,
	pub anchors: AnchorService,
	pub search: SearchService,
	pub analysis: AnalysisService,
}
impl<R: Runtime> EstateEngine<R> {
	pub fn new(runtime: R) -> Result<Self> {
		// let state = EstateState::load_from_disk().unwrap();
		// let state_monitor = StateMonitor::new(&state_path)?;
		Ok(Self {
			// state,
			// state_monitor,
			runtime: Arc::new(runtime),
			estate: Estate::default(),
			workspace: Workspace::new(),
			registry: EstateRegistry::default(),
			index: EstateIndex::default(),
			resolver: EstateResolver::default(),
			graph: EstateGraph::default(),
			discovery: EstateDiscovery::default(),
			vfs: EstateVfs::default(),
			anchors: AnchorService::default(),
			search: SearchService::default(),
			analysis: AnalysisService::default(),
		})
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateRegistry;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateIndex;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateResolver;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateGraph;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateDiscovery;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct EstateVfs;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct AnchorService {
	registry: EstateRegistry,
	index: EstateIndex,
	resolver: EstateResolver,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct SearchService;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct AnalysisService;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) enum ReferenceKind {
	#[default]
	File,
	Link,
	Embed,
	Relative,
	// potentially:
	Anchor,
	Asset,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct Reference<'a> {
	pub target: &'a str,
	pub fragment: Option<&'a str>,
	pub kind: ReferenceKind,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) struct ResolveContext {
	pub scope: EstateScope,
	pub from: Uuid,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) enum EstateScope {
	System,
	User,
	#[default]
	Workspace,
}
