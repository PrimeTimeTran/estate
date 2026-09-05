use crate::prelude::*;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct EstateDiscovery;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(Clone, Debug)]

pub struct EstateDiscovery<State = Disconnected> {
	pub store: DiscoveryStore,
	pub state: State,
}
