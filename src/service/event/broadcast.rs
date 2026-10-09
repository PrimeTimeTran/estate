use crate::prelude::*;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct BroadcastReceiver<T> {
	pub id: u64,
	pub owner: &'static str,
	pub rx: broadcast::Receiver<T>,
}

impl ReceivesEvents for BroadcastReceiver<e::Event> {
	fn try_recv(&mut self) -> Option<e::Event> {
		match self.rx.try_recv() {
			Ok(event) => {
				tracing::debug!(?event.kind, "ScreenInstance::event");
				Some(event)
			}

			Err(TryRecvError::Empty) => {
				tracing::debug!(id = self.id, "RECEIVER EMPTY");
				None
			}

			Err(err) => {
				tracing::error!(id = self.id, ?err, "RECEIVER ERROR");
				None
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct BroadcastSender<T> {
	pub tx: broadcast::Sender<T>,
}

impl BroadcastSender<e::Event> {
	pub fn new(tx: broadcast::Sender<e::Event>) -> Self {
		Self { tx }
	}
}

impl SendsEvents for BroadcastSender<e::Event> {
	fn send(&self, event: e::Event) {
		tracing::info!(event = ?event, "🚌 SendsEvents SEND");
		let _ = self.tx.send(event);
	}
}
