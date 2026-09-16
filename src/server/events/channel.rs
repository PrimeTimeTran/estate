use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{
	Receiver as TokioReceiver, Sender as TokioSender, channel as tokio_channel,
	error::{SendError, TrySendError},
};

use crate::prelude::*;

pub fn channel<T>(capacity: usize) -> (EventSender<T>, EventReceiver<T>) {
	// EventSender<T> ─┐
	// EventSender<T> ─┼──> EventReceiver<T>
	// EventSender<T> ─┘
	// Multi Producer Single Consumer
	let (tx, rx) = tokio_channel(capacity);
	(EventSender { tx }, EventReceiver { rx })
}

impl<T> Clone for EventSender<T> {
	fn clone(&self) -> Self {
		Self {
			tx: self.tx.clone(),
		}
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

impl<T> EventSender<T> {
	pub async fn send(&self, event: T) -> Result<(), SendError<T>> {
		self.tx.send(event).await
	}
	pub fn try_send(&self, event: T) -> Result<(), TrySendError<T>> {
		self.tx.try_send(event)
	}
	pub fn blocking_send(&self, event: T) -> Result<(), SendError<T>> {
		self.tx.blocking_send(event)
	}
}

#[derive(Debug)]
pub struct EventSender<T> {
	tx: TokioSender<T>,
}

#[derive(Debug)]
pub struct EventReceiver<T> {
	rx: TokioReceiver<T>,
}

// pub struct NativeEventReceiver {
// 	rx: tokio::sync::broadcast::Receiver<e::Event>,
// }

// impl NativeEventReceiver {
// 	pub fn try_recv(&mut self) -> Result<e::Event, tokio::sync::broadcast::error::TryRecvError> {
// 		self.rx.try_recv()
// 	}
// }
