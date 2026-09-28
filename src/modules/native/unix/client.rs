use crate::prelude::*;

pub struct DaemonClient {
	PATH_SOCKET: &'static str,
}
impl DaemonClient {
	pub fn new() -> Self {
		Self {
			PATH_SOCKET: PATH_SOCKET,
		}
	}
	pub async fn execute(&self, action: ActionRequest) -> Result<DaemonResponse> {
		let mut stream = UnixStream::connect(self.PATH_SOCKET).await?;
		let request = serde_json::to_string(&action)?;
		stream.write_all(request.as_bytes()).await?;
		stream.write_all(b"\n").await?;
		let mut buf = Vec::new();
		stream.read_to_end(&mut buf).await?;
		let response: DaemonResponse = serde_json::from_slice(&buf)?;
		Ok(response)
	}
}
