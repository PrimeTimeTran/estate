use crate::prelude::{traits::Ctx, *};

#[cfg(target_arch = "wasm32")]
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
			_ctx: PhantomData,
		}
	}

	#[cfg(target_arch = "wasm32")]
	pub fn new(cancel: CancellationToken) -> Self {
		Self {
			cancel,
			_ctx: PhantomData,
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
	#[cfg(target_arch = "wasm32")]
	_phantom: PhantomData<J>,
	_ctx: PhantomData<C>,
}

#[cfg(target_arch = "wasm32")]
pub type PlatformJoin = ();

#[cfg(not(target_arch = "wasm32"))]
pub type ClockWork<NativeContext> = WorkHandle<NativeContext, tokio::task::JoinHandle<()>>;

#[cfg(target_arch = "wasm32")]
pub type ClockWork<WebContext> = WorkHandle<WebContext, ()>;
