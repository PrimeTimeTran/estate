use crate::prelude::*;
use crate::{data, prelude::*};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub struct EstateDiscovery;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(Clone, Debug)]
pub struct EstateDiscovery {
	pub store: DiscoveryStore,
	pub task_tx: mpsc::Sender<DiscoveryTask>,
}
