use crate::prelude::{traits::Ctx, *};

impl<C> WorkHandle<C, std::thread::JoinHandle<()>>
where
	C: Ctx,
{
	pub fn join(self) -> std::thread::Result<()> {
		self.join()
	}
}
#[cfg(not(target_arch = "wasm32"))]
impl<C> WorkHandle<C, tokio::task::JoinHandle<()>>
where
	C: Ctx,
{
	pub async fn join(self) -> Result<JoinHandle<()>> {
		Ok(self.join)
	}
}
impl<C, J> WorkHandle<C, J>
where
	C: Ctx,
{
	#[cfg(not(target_arch = "wasm32"))]
	pub fn new(cancel: CancellationToken, join: J) -> Self {
		Self {
			cancel,
			join,
			_phantom: PhantomData,
		}
	}

	#[cfg(target_arch = "wasm32")]
	pub fn new(cancel: CancellationToken) -> Self {
		Self {
			cancel,
			_phantom: PhantomData,
		}
	}

	pub fn stop(&self) {
		self.cancel.cancel();
	}
}

/// "This is a unit of work that I know how to stop."
///
pub struct WorkHandle<C, J>
where
	C: Ctx,
{
	pub cancel: CancellationToken,
	#[cfg(not(target_arch = "wasm32"))]
	pub join: J,
	_phantom: PhantomData<(C, J)>,
}
