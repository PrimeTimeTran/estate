import type { TauriContext as TauriContextType } from "./types";

declare global {
  type TauriContext = TauriContextType;
}

export {};
