use anyhow::Context;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_foundation::NSString;

use crate::{ipc::IpcServer, model::resolver::crate_root, prelude::*};
use std::process::Command;

use axuielement::prelude::*;
async fn handle_event_client(mut stream: tokio::net::UnixStream, events: EventBus) -> Result<()> {
	let mut rx = events.subscribe();
	loop {
		let event = rx.recv().await?;
		let json = serde_json::to_string(&event)?;
		stream.write_all(json.as_bytes()).await?;
		stream.write_all(b"\n").await?;
	}
}
fn print_focused_app() -> Result<(), Box<dyn std::error::Error>> {
	let Some(system) = system_wide() else {
		eprintln!("no system-wide accessibility object");
		return Ok(());
	};
	println!("api_enabled = {}", api_enabled());
	println!("is_trusted  = {}", is_process_trusted());
	if let Some(app) = system.focused_application()? {
		println!("focused pid   = {}", app.pid()?);
		println!("focused attrs = {:?}", app.attribute_names()?);
	}
	if let Some(focused) = system.focused_ui_element()? {
		println!(
			"role  = {:?}",
			focused.string_attribute(axuielement::ax_attribute::AX_ROLE_ATTRIBUTE)?
		);
		println!(
			"title = {:?}",
			focused.string_attribute(axuielement::ax_attribute::AX_TITLE_ATTRIBUTE)?
		);
		println!("actions = {:?}", focused.action_names()?);
	}
	Ok(())
}

impl<C> App<C>
where
	C: Ctx,
{
	pub fn init_daemon(&mut self) -> Result<()> {
		tracing::info!("init_daemon");
		Ok(())
	}
}

impl<C: Ctx> Host<C> {
	pub fn start(&mut self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		self.start_ipc()?;
		// let ipc = IpcServer::new(PathBuf::from("/tmp/estate.sock"));
		// ipc.init(&self.worker)?;
		self.start_watchers()?;
		// self.start_grpc_server()?;
		Ok(self.start_hid_bridge()?)
	}
	pub fn start_ipc(&self) -> Result<()> {
    let ipc = IpcServer::new(PathBuf::from("/tmp/estate.sock"));
	
    self.worker.run_background(|_cancel| async move {
        if let Err(error) = ipc.start().await {
            tracing::error!(%error, "Estate IPC server stopped");
        }
    });
	
    Ok(())
	}
	pub fn start_watchers(&mut self) -> Result<()> {
		// cargo.toml, settings files, caches, index,
		Ok(())
	}

	fn start_grpc_server(&self) -> Result<()> {
		let handle = self.worker.start_grpc_server();
		// retain handle
		Ok(())
	}
	pub fn start_hid_bridge(&self) -> Result<WorkHandle<C, tokio::task::JoinHandle<()>>> {
		let mut hid = MacosHid::new()?;
		hid.start();
		let events = self.event_bus.clone();
		let handle = self.worker.run_background(move |cancel| async move {
			if let Err(error) = hid.run(events, cancel).await {
				tracing::error!(%error, "macOS HID stopped");
			}
		});
		Ok(handle)
	}

	pub fn new(context: Arc<C>, tokio: tokio::runtime::Runtime) -> anyhow::Result<Self> {
		// ## TODO:
		//
		// - [ ] Read settings.json using settings_resolver
		// - [ ] Drive behavior using settings

		let handle = tokio.handle().clone();
		let event_bus = EventBus::new();
		let runtime = NativeRuntime::new(Arc::clone(&context), handle.clone(), event_bus.clone())?;
		runtime.start_dispatcher();
		Ok(Self {
			context,
			runtime,
			event_bus,
			worker: HostWorker::new(),
			clock: HostClock::new(handle),
			tokio,
		})
	}
}
