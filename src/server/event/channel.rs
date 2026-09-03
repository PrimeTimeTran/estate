use tokio::sync::mpsc::error::TryRecvError;

use crate::{e, prelude::*};

pub fn channel<T>(capacity: usize) -> (EventSender<T>, EventReceiver<T>) {
	// EventSender<T> ─┐
	// EventSender<T> ─┼──> EventReceiver<T>
	// EventSender<T> ─┘
	// Multi Producer Single Consumer
	let (tx, rx) = tokio::sync::mpsc::channel(capacity);
	(EventSender { tx }, EventReceiver { rx })
}

#[derive(Debug)]
pub struct EventSender<T> {
	tx: tokio::sync::mpsc::Sender<T>,
}

#[derive(Debug)]
pub struct EventReceiver<T> {
	rx: tokio::sync::mpsc::Receiver<T>,
}

impl<T> Clone for EventSender<T> {
	fn clone(&self) -> Self {
		Self {
			tx: self.tx.clone(),
		}
	}
}

impl<T> EventSender<T> {
	pub async fn send(&self, event: T) -> Result<(), mpsc::error::SendError<T>> {
		self.tx.send(event).await
	}

	pub fn try_send(&self, event: T) -> Result<(), mpsc::error::TrySendError<T>> {
		self.tx.try_send(event)
	}

	pub fn blocking_send(&self, event: T) -> Result<(), mpsc::error::SendError<T>> {
		self.tx.blocking_send(event)
	}
}

impl<T> EventReceiver<T> {
	/// Consume one event if available.
	pub fn poll(&mut self) -> Option<T> {
		self.rx.try_recv().ok()
	}

	/// Consume every currently available event.
	pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
		std::iter::from_fn(|| self.rx.try_recv().ok())
	}

	pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
		self.rx.try_recv()
	}
}

pub struct NativeEventReceiver {
	rx: tokio::sync::broadcast::Receiver<e::Event>,
}

impl NativeEventReceiver {
	pub fn try_recv(&mut self) -> Result<e::Event, tokio::sync::broadcast::error::TryRecvError> {
		self.rx.try_recv()
	}
}
