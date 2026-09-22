pub trait Observer {
	type Event;

	fn subscribe(&self) -> EventStream<Self::Event>;
}

pub trait BrowserObserver {
	fn browsers(&self) -> &[Browser];
	fn tabs(&self, browser: BrowserId) -> Vec<Tab>;
	fn subscribe(&self) -> EventStream<BrowserEvent>;
}

#[derive(Clone, Debug)]
pub struct Browser {
	pub id: BrowserId,
	pub name: String,
	pub process_id: u32,
}

#[derive(Clone, Debug)]
pub struct Tab {
	pub id: TabId,
	pub window_id: WindowId,
	pub url: Url,
	pub title: Option<String>,
	pub active: bool,
}

#[derive(Clone, Debug)]
pub enum BrowserEvent {
	BrowserOpened(Browser),
	BrowserClosed(BrowserId),

	TabCreated(Tab),
	TabClosed(TabId),

	TabActivated {
		tab_id: TabId,
	},

	TabUpdated {
		tab_id: TabId,
		changes: TabChanges,
	},

	Navigation {
		tab_id: TabId,
		from: Option<Url>,
		to: Url,
	},
}

pub enum EventSource {
	ChromeExtension,
	MacosAccessibility,
	ProcessTree,
	Filesystem,
	Application,
}

pub enum Event {
	Application(ApplicationEvent),
	Browser(BrowserEvent),
	Window(WindowEvent),
	Document(DocumentEvent),
	Process(ProcessEvent),
}
