use crate::prelude::*;

impl Default for EventBus {
	fn default() -> Self {
		Self::new()
	}
}

impl EventBus {
	pub fn new() -> Self {
		let (tx, _) = broadcast::channel(256);
		tracing::info!("EventBus CREATED");

		Self { tx }
	}

	/// Raw Tokio broadcast receiver.
	pub fn subscribe(&self) -> broadcast::Receiver<e::Event> {
		tracing::info!("EventBus subscribe RAW");
		self.tx.subscribe()
	}

	/// Application-level event receiver.
	pub fn subscribe_broadcast(&self, owner: &'static str) -> BroadcastReceiver<e::Event> {
		let id = NEXT_RECEIVER_ID.fetch_add(1, Ordering::Relaxed);

		tracing::info!(id, owner, "EventBus creating broadcast receiver");

		BroadcastReceiver {
			id,
			owner,
			rx: self.tx.subscribe(),
		}
	}

	/// Application-level event sender.
	pub fn sender(&self) -> BroadcastSender<e::Event> {
		BroadcastSender {
			tx: self.tx.clone(),
		}
	}
}

impl EventService {
	pub fn new(events: EventBus) -> Self {
		Self { events }
	}
}
