use crate::app::modules::runtime::Runtime;
use crate::data::defaults::{self, INDEX_PATH};
use crate::native::session::Session;
use crate::{
	app::{
		task::{self},
		*,
	},
	event::{self, EventKind},
	prelude::*,
};
// Events = n that happened
// Handlers = reactions to facts
// Tasks = units of work
// Commands = requests to do something
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
	async fn handle(&self, event: &Event, runtime: &NativeRuntime);
}
pub struct TaskHandler;
#[async_trait::async_trait]
impl EventHandler for TaskHandler {
	async fn handle(&self, event: &Event, runtime: &NativeRuntime) {
		tracing::debug!("📡 EventHandler.handle {:?}", event);
		let event::EventKind::TaskRequested { request } = &event.kind else {
			return;
		};
		let task_id = match request {
			TaskRequest::Create(kind) => {
				tracing::debug!("TaskRequest::Create {:?}", kind);
				let mut tasks = runtime.tasks.write().unwrap();
				let task_id = tasks.create(kind.clone());
				runtime.emit(Event::daemon(EventKind::TaskCreated {
					task_id,
					kind: kind.clone(),
				}));
				match kind {
					TaskKind::SessionStart => {
						tracing::debug!("🔥 SessionStart");
						runtime.emit(Event::daemon(EventKind::TaskRequested {
							request: TaskRequest::Create(TaskKind::LoadMaster),
						}));
						runtime.emit(Event::daemon(EventKind::TaskRequested {
							request: TaskRequest::Create(TaskKind::RebuildIndex),
						}));
						runtime.emit(Event::daemon(EventKind::TaskRequested {
							request: TaskRequest::Create(TaskKind::IndexWorkspace),
						}));
					}
					TaskKind::SessionStop => {
						tracing::debug!("🛑 SessionStop");
						// let session = runtime.session.clone();
						// Master::save(session);
					}
					_ => {}
				}
				task_id
			}
			TaskRequest::Run(task_id) => *task_id,
			_ => {
				return;
			}
		};
		let task = {
			let tasks = runtime.tasks.read().unwrap();
			let Some(task) = tasks.get(task_id).cloned() else {
				tracing::warn!(%task_id, "requested task not found");
				return;
			};
			task
		};
		{
			let mut tasks = runtime.tasks.write().unwrap();
		}
		runtime.emit(Event::daemon(event::EventKind::TaskStarted { task_id }));
		let runtime = runtime.clone();
		tokio::spawn(async move {
			tracing::debug!(
				%task_id,
				task = %task.name,
				"task starting"
			);
			match TaskRunner::execute(&runtime, task.clone()).await {
				Ok(()) => {
					tracing::debug!("TaskHandler match TaskRunner::execute {:?}", task);
					runtime.emit(Event::daemon(event::EventKind::TaskCompleted { task_id }));
				}
				Err(error) => {
					runtime.emit(Event::daemon(event::EventKind::TaskFailed {
						task_id,
						error: error.to_string(),
					}));
				}
			}
		});
	}
}

pub struct LogHandler;
#[async_trait::async_trait]
impl EventHandler for LogHandler {
	async fn handle(&self, event: &Event, _runtime: &NativeRuntime) {
		tracing::debug!("📡 received {:?}", event);
	}
}
pub struct FileWatcherHandler;
#[async_trait::async_trait]
impl EventHandler for FileWatcherHandler {
	async fn handle(&self, event: &Event, runtime: &NativeRuntime) {
		if let event::EventKind::FileModified { inode, path } = &event.kind {
			tracing::debug!("📡 FileWatcherHandler handle {:?} ({:?})", event, inode);
			runtime.emit(Event::daemon(event::EventKind::IndexUpdated {
				files_changed: 1,
			}));
		}
	}
}
pub struct StateHandler;
#[async_trait::async_trait]
impl EventHandler for StateHandler {
	async fn handle(&self, event: &Event, runtime: &NativeRuntime) {
		tracing::debug!("🔥 StateHandler received: {:?}", event.kind);
		let snapshot = {
			let mut state = runtime.state.write();
			match &event.kind {
				event::EventKind::DaemonStarted => {}
				event::EventKind::DaemonStarted => {
					state.starts += 1;
					state.status_checks += 1;
					state.started_at = event.timestamp;
				}
				event::EventKind::StatusRequested => {
					state.status_checks += 1;
				}
				event::EventKind::IndexUpdated { files_changed } => {
					state.files_indexed += files_changed;
				}
				event::EventKind::TaskCreated { task_id, kind } => {
					state.tasks_created += 1;
					state.jobs.push_back(Job {
						id: *task_id,
						task_id: *task_id,
						kind: kind.to_owned(),
						status: JobStatus::Pending,
						created_at: event.timestamp,
						started_at: None,
						completed_at: None,
					});
				}
				event::EventKind::TaskStarted { task_id } => {
					if let Some(job) = state.jobs.iter_mut().find(|job| job.id == *task_id) {
						job.status = JobStatus::Running;
						job.started_at = Some(event.timestamp);
					}
				}
				event::EventKind::TaskCompleted { task_id } => {
					state.tasks_completed += 1;
					if let Some(job) = state.jobs.iter_mut().find(|job| job.id == *task_id) {
						job.status = JobStatus::Completed;
						job.completed_at = Some(event.timestamp);
					}
				}
				event::EventKind::TaskFailed { task_id, error } => {
					if let Some(job) = state.jobs.iter_mut().find(|job| job.id == *task_id) {
						job.status = JobStatus::Failed;
						job.completed_at = Some(event.timestamp);
					}
				}
				event::EventKind::DaemonStopped => {
					let run_duration = event.timestamp.saturating_sub(state.started_at);
					state.longest_run = state.longest_run.max(run_duration);
				}
				_ => {}
			}
			state.events_processed += 1;
			state.revision += 1;
			state.clone()
		};
		// Ok(runtime.save(&snapshot)?)
		runtime.save(&snapshot);
	}
}
pub struct CommandHandler;
#[async_trait::async_trait]
impl EventHandler for CommandHandler {
	async fn handle(&self, event: &Event, runtime: &NativeRuntime) {
		tracing::debug!("CommandHandler handler {:?}", event);

		match &event.kind {
			EventKind::SessionStop { .. } => {
				tracing::debug!("ABOUT TO SAVE SESSION");

				let mut session = runtime.session.clone();
				let serialized = session.end_session();
				match Master::save(serialized).await {
					Ok(()) => {
						tracing::debug!("SESSION SAVED");
					}
					Err(err) => {
						tracing::error!("SESSION SAVE FAILED: {err:#}");
					}
				}
			}

			EventKind::CommandExecuted { command } => {
				// handle actual commands here
			}

			_ => {}
		}
	}
}
// 			"task_create" => {
// 				tracing::info!("CommandHandler task_create {:?}", event);
// 				runtime.emit(Event::app(event::EventKind::TaskRequested {
// 					request: TaskRequest::Create(TaskKind::SyncBookmarks),
// 				}));
// 			}
// 			"task_list" => {
// 				let tasks = runtime.tasks.read().unwrap();
// 				println!("════════════════════════════════════");
// 				println!("             ESTATE TASKS");
// 				println!("════════════════════════════════════");
// 				if tasks.count() == 0 {
// 					println!("No tasks in memory.");
// 				} else {
// 					for task in tasks.list() {
// 						println!("[{}] {} — {:?}", task.id, task.name, task.status);
// 					}
// 				}
// 				drop(tasks);
// 				let state = EstateState::load_from_disk().unwrap();
// 				println!();
// 				println!("──────────── persisted state ────────────");
// 				println!("starts:           {}", state.starts);
// 				println!("status checks:    {}", state.status_checks);
// 				println!("tasks completed:  {}", state.tasks_completed);
// 				println!("files indexed:    {}", state.files_indexed);
// 				println!("events processed: {}", state.events_processed);
// 				println!("longest run:      {}", state.longest_run);
// 				println!("started at:       {}", state.started_at);
// 				runtime.emit(Event::daemon(event::EventKind::StatusRequested));
// 			}
// 			"task_clear" => {
// 				{
// 					let mut tasks = runtime.tasks.write().unwrap();
// 					tasks.clear();
// 				}
// 				runtime.emit(Event::daemon(event::EventKind::TasksCleared));
// 			}
// 			"dev_info" => {
// 				runtime.emit(Event::daemon(event::EventKind::TaskRequested {
// 					request: TaskRequest::Create(TaskKind::BuildEstatePrototype),
// 				}));
// 			}
// 			"rebuild_index" => {
// 				runtime.emit(Event::daemon(event::EventKind::TaskRequested {
// 					request: TaskRequest::Create(TaskKind::RebuildIndex),
// 				}));
// 			}
// 			"sync_bookmarks" => {
// 				runtime.emit(Event::daemon(event::EventKind::TaskRequested {
// 					request: TaskRequest::Create(TaskKind::SyncBookmarks),
// 				}));
// 			}
// 			"generate_dashboard" => {
// 				runtime.emit(Event::daemon(event::EventKind::TaskRequested {
// 					request: TaskRequest::Create(TaskKind::GenerateView("dashboard".into())),
// 				}));
// 			}
// 			_ => {
// 				println!("⚠️ unknown command: {command}");
// 			}
// 		}
// 	}
// }
pub struct TaskRunner;
impl TaskRunner {
	pub async fn execute(runtime: &NativeRuntime, task: Task) -> Result<()> {
		tracing::info!("TaskRunner execute {:?}", task);
		match task.kind {
			TaskKind::SessionStart => {
				tracing::debug!("SessionStart");
				tracing::debug!("✅ LoadMaster complete");
			}
			TaskKind::SessionStop => {
				tracing::debug!("SessionStop");
				tracing::debug!("✅ LoadMaster complete");
			}
			TaskKind::LoadMaster => {
				tracing::debug!("LoadMaster");
				tokio::time::sleep(std::time::Duration::from_secs(1)).await;
				tracing::debug!("✅ LoadMaster complete");
			}
			TaskKind::IndexWorkspace => {
				let started = std::time::Instant::now();
				tracing::info!("Index Timer Start 🏁 {:?}ms", started);

				let mut discovery = tokio::task::spawn_blocking(EstateDiscovery::init)
					.await
					.map_err(|error| anyhow::anyhow!("discovery task panicked: {error}"))??;

				discovery.write_discovery_result()?;

				let duration = started.elapsed().as_millis();

				runtime.emit(Event::daemon(EventKind::WorkspaceIndexed {
					duration: duration as u64,
				}));

				tracing::info!("Index Time End ⏰ {}ms", duration);
				tracing::info!("Files: {}", discovery.files.len());
				tracing::info!("Types: {}", discovery.types().len());
			}
			TaskKind::RebuildIndex => {
				tracing::info!("🔨 rebuilding index");
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				tracing::info!("✅ index rebuild complete");
			}
			TaskKind::GenerateView(name) => {
				tracing::info!("👁️ generating view: {name}");
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				tracing::info!("✅ view generated: {name}");
			}
			TaskKind::SyncBookmarks => {
				tracing::info!("🔖 TaskKind::SyncBookmarks {:?}", task);
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
				tracing::info!("✅ bookmark sync complete {:?}", task.id);
			}
			TaskKind::BuildEstatePrototype => {
				tracing::info!("🚧 starting BuildEstatePrototype");
				for i in 1..=10 {
					tokio::time::sleep(std::time::Duration::from_secs(1)).await;
					tracing::info!("🚧 prototype task: {i}/10");
				}
				tracing::info!("✅ BuildEstatePrototype complete");
			}
		}
		Ok(())
	}
}
#[derive(Debug, Clone)]
pub struct AppHandler;
#[async_trait::async_trait]
impl EventHandler for AppHandler {
	async fn handle(&self, event: &Event, runtime: &NativeRuntime) {
		if !matches!(event.kind, EventKind::SessionStart) {
			return;
		}
		match event.kind.clone() {
			event::EventKind::SessionStop { session } => {
				tracing::info!("🛑 SessionStop");
			}
			event::EventKind::SessionStart => {}
			_ => {
				println!("not interested")
			}
		}
		tracing::info!("🔥 SessionStart → SessionStart");
		runtime.emit(Event::daemon(EventKind::TaskRequested {
			request: TaskRequest::Create(TaskKind::SessionStart),
		}));
	}
}
pub struct Master;
impl Master {
	pub async fn save(session: Value) -> anyhow::Result<()> {
		let path = dirs::home_dir()
			.ok_or_else(|| anyhow::anyhow!("could not determine home directory"))?
			.join(INDEX_PATH);
		let contents = tokio::fs::read_to_string(&path).await?;
		let mut master: serde_json::Value = serde_json::from_str(&contents)?;
		master
			.pointer_mut("/logs/sessions")
			.and_then(serde_json::Value::as_array_mut)
			.ok_or_else(|| anyhow::anyhow!("logs.sessions is not an array"))?
			.push(serde_json::to_value(session)?);

		tokio::fs::write(&path, serde_json::to_string_pretty(&master)?).await?;

		Ok(())
	}
}
