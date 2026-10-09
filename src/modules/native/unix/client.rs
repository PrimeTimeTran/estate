use crate::prelude::*;

pub struct DaemonClient {
	ESTATE_DAEMON_SOCKET: &'static str,
}
impl DaemonClient {
	pub fn new() -> Self {
		Self {
			ESTATE_DAEMON_SOCKET: ESTATE_DAEMON_SOCKET,
		}
	}
	pub async fn execute(&self, action: ActionRequest) -> Result<DaemonResponse> {
		let mut stream = UnixStream::connect(self.ESTATE_DAEMON_SOCKET).await?;
		let request = serde_json::to_string(&action)?;
		stream.write_all(request.as_bytes()).await?;
		stream.write_all(b"\n").await?;
		let mut buf = Vec::new();
		stream.read_to_end(&mut buf).await?;
		let response: DaemonResponse = serde_json::from_slice(&buf)?;
		Ok(response)
	}
}
