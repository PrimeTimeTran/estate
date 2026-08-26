use crate::app::app::{ App, Runtime };

pub trait AppHost<R: Runtime> {
	fn app(&mut self) -> &mut App<R>;
}
