import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { EstateContext, EstateEventEnvelope, KeyboardKeyEvent, MAC_KEY_DATA_BY_CODE, TauriContext } from "./types";

const activeModifierCodes = new Set<number>();

function emitKeyboardKeyEvent(type: KeyboardKeyEvent["type"], keyCode: number) {
  const key = MAC_KEY_DATA_BY_CODE[keyCode];

  if (key) {
    window.dispatchEvent(
      new CustomEvent<KeyboardKeyEvent>("estate-key-event", {
        detail: { type, key },
      }),
    );
  }
}

function toggleKeyboardModifier(keyCode: number) {
  const isDown = !activeModifierCodes.has(keyCode);

  if (isDown) {
    activeModifierCodes.add(keyCode);
  } else {
    activeModifierCodes.delete(keyCode);
  }

  emitKeyboardKeyEvent(isDown ? "key_down" : "key_up", keyCode);
}
/* -------------------------------------------------------------------------- */
/* Helpers                                                                    */
/* -------------------------------------------------------------------------- */

function setStatus(message: string) {
  document.querySelector("#status")!.textContent = message;
}

function setText(id: string, value: string) {
  const element = document.querySelector(`#${id}`);

  if (element) {
    element.textContent = value;
  }
}

/* -------------------------------------------------------------------------- */
/* Tauri context                                                              */
/* -------------------------------------------------------------------------- */

async function loadTauriContext() {
  console.log("🔥 FRONTEND → invoke tauri_context");

  const context = await invoke<TauriContext>("tauri_context");

  console.log("🔥 FRONTEND ← Tauri:", context);

  setText("tauri-pid", String(context.pid));
  setText("tauri-platform", context.platform);
  setText("tauri-arch", context.arch);
  setText("tauri-cwd", context.cwd);

  return context;
}

/* -------------------------------------------------------------------------- */
/* Estate context                                                             */
/* -------------------------------------------------------------------------- */

async function loadEstateContext() {
  console.log("🔥 FRONTEND → invoke estate_context");

  const context = await invoke<EstateContext>("estate_context");

  console.log("🔥 FRONTEND ← Estate:", context);

  setText("estate-connection", context.connection_id);
  setText("active-app", context.active_app);
  setText("workspace", context.workspace ?? "—");
  setText("mode", context.mode);

  return context;
}

/* -------------------------------------------------------------------------- */
/* Estate events                                                              */
/* -------------------------------------------------------------------------- */

async function listenForEstateEvents() {
  console.log("🔥 FRONTEND → listening for Estate events");

  await listen<EstateEventEnvelope>("estate-event", (event) => {
    console.log("🔥 FRONTEND ← Estate event:", event.payload);

    handleEstateEvent(event.payload);
  });
}

async function loadEverything() {
  setStatus("Loading Tauri context...");
  await loadTauriContext();
  setStatus("Loading Estate context...");
  await loadEstateContext();
  setStatus("✓ Connected to Estate");
}

function handleEstateEvent(envelope: EstateEventEnvelope) {
  const event = envelope.event;
  if ("ActiveAppChanged" in event) {
    const app = event.ActiveAppChanged;
    setText("active-app", app.name);
    setText("last-event", `ACTIVE APP — ${app.name}`);
    console.log("🔁 Active app:", app.name, app.bundle_id, app.pid);
    return;
  }

  if ("KeyDown" in event) {
    const { key_code } = event.KeyDown;
    setText("last-event", `KEY DOWN — code ${key_code}`);
    emitKeyboardKeyEvent("key_down", key_code);
    return;
  }

  if ("KeyUp" in event) {
    const { key_code } = event.KeyUp;
    setText("last-event", `KEY UP — code ${key_code}`);
    emitKeyboardKeyEvent("key_up", key_code);
    return;
  }

  if ("FlagsChanged" in event) {
    const { key_code } = event.FlagsChanged;
    setText("last-event", `FLAGS CHANGED — code ${key_code}`);
    toggleKeyboardModifier(key_code);
  }
}
/* -------------------------------------------------------------------------- */
/* Startup                                                                    */
/* -------------------------------------------------------------------------- */

window.addEventListener("DOMContentLoaded", async () => {
  console.log("🔥 FRONTEND READY");

  try {
    await listenForEstateEvents();
    await loadEverything();
  } catch (error) {
    console.error("🔥 FRONTEND INITIALIZATION FAILED:", error);

    setStatus(`Initialization failed: ${String(error)}`);
  }
});
