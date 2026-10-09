export type TauriContext = {
	pid: number
	platform: string
	arch: string
	cwd: string
}

export type EstateContext = {
	connection_id: string
	active_app: string
	workspace: string | null
	mode: string
}

export type EstateEvent =
	| {
			KeyDown: { key_code: number }
	  }
	| {
			KeyUp: { key_code: number }
	  }
	| {
			FlagsChanged: { key_code: number }
	  }
	| {
			ActiveAppChanged: {
				name: string
				bundle_id: string
				pid: number
			}
	  }

export type EstateEventEnvelope = {
	event: EstateEvent
}

export type KeyboardKeyEvent = {
	type: 'key_down' | 'key_up'
	key: string
}

export const MAC_KEY_DATA_BY_CODE: Record<number, string> = {
	0x00: 'A',
	0x01: 'S',
	0x02: 'D',
	0x03: 'F',
	0x04: 'H',
	0x05: 'G',
	0x06: 'Z',
	0x07: 'X',
	0x08: 'C',
	0x09: 'V',
	0x0b: 'B',
	0x0c: 'Q',
	0x0d: 'W',
	0x0e: 'E',
	0x0f: 'R',
	0x10: 'Y',
	0x11: 'T',
	0x12: '1',
	0x13: '2',
	0x14: '3',
	0x15: '4',
	0x16: '6',
	0x17: '5',
	0x18: '=',
	0x19: '9',
	0x1a: '7',
	0x1b: '-',
	0x1c: '8',
	0x1d: '0',
	0x1e: ']',
	0x1f: 'O',
	0x20: 'U',
	0x21: '[',
	0x22: 'I',
	0x23: 'P',
	0x24: 'ENTER',
	0x25: 'L',
	0x26: 'J',
	0x27: "'",
	0x28: 'K',
	0x29: ';',
	0x2a: '\\\\',
	0x2b: ',',
	0x2c: '/',
	0x2d: 'N',
	0x2e: 'M',
	0x2f: '.',
	0x32: 'BACKTICK',
	0x30: 'TAB',
	0x31: 'SPACE',
	0x33: 'DELETE',
	0x35: 'ESC',
	0x36: 'RCMD',
	0x37: 'LCMD',
	0x38: 'LSHIFT',
	0x39: 'CAPS',
	0x3a: 'LOPT',
	0x3b: 'LCTRL',
	0x3c: 'RSHIFT',
	0x3d: 'ROPT',
	0x3e: 'RCTRL',
	0x3f: 'FN',
	0x60: 'F5',
	0x61: 'F6',
	0x62: 'F7',
	0x63: 'F3',
	0x64: 'F8',
	0x65: 'F9',
	0x67: 'F11',
	0x6d: 'F10',
	0x6f: 'F12',
	0x76: 'F4',
	0x75: 'DELETE',
	0x78: 'F2',
	0x7a: 'F1',
	0x7b: 'ARROWLEFT',
	0x7c: 'ARROWRIGHT',
	0x7d: 'ARROWDOWN',
	0x7e: 'ARROWUP',
}

export type EstateNavigationState = {
	/**
	 * Which navigation surfaces are currently visible.
	 */
	ui: {
		topTabs: boolean
		sidebar: boolean
		searchBar: boolean
	}

	/**
	 * Current location inside Estate.
	 */
	current: {
		tab: string
		route: string
		query: string
	}

	/**
	 * Open/recent top-level tabs.
	 */
	tabs: {
		open: string[]
		active: string

		/**
		 * Browser-style navigation history.
		 */
		history: {
			back: string[]
			forward: string[]
		}
	}

	/**
	 * Sidebar navigation state.
	 */
	sidebar: {
		active: string
		collapsed: boolean
	}

	/**
	 * Search state + search history.
	 */
	search: {
		query: string

		history: string[]

		/**
		 * Optional currently selected result.
		 */
		selected: string | null
	}

	/**
	 * Actions the user has performed.
	 *
	 * Useful later for command palette, undo/replay,
	 * analytics, AI context, etc.
	 */
	actions: {
		history: Array<{
			id: string
			type: string
			name: string
			timestamp: number
			payload?: Record<string, unknown>
		}>
	}

	/**
	 * General navigation preferences.
	 */
	settings: {
		maxHistory: number
		maxSearchHistory: number
	}
}
