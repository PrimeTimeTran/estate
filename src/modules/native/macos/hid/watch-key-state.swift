// MARK: - Modifier state tracking
// CGEvent flags are aggregate:
//   maskCommand = "some Command is down"
// They do NOT directly tell us:
//   "left Command is down"
// We therefore reconstruct left/right state from flagsChanged
// keycodes. This is useful diagnostic state, but it is NOT raw HID
// truth.
// Return the state of the *specific modifier key* represented by keyCode.
// IMPORTANT:
// Do NOT infer this from whether the aggregate Shift/Option/etc. flag
// changed. Another physical modifier of the same family may already
// be holding that aggregate flag.
//
// Instead, compare the event's aggregate flag against the previous
// per-key state we have recorded.

// let initialTimer = Timer.scheduledTimer(withTimeInterval: initialDelay, repeats: false) { _ in
//   let checkState = {
//     let status = isControlKeyPressed() ? "DOWN" : "UP"
//     print("[\(getCurrentTimestamp())] Control key is \(status).")
//     fflush(stdout)
//   }
//
//   // Execute the initial synchronized check immediately
//   checkState()
//
//   // Establish the permanent recurring timer
//   let repeatingTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { _ in
//     checkState()
//   }
//
//   // Attach the repeating timer to the CoreFoundation RunLoop
//   RunLoop.current.add(repeatingTimer, forMode: .default)
// }

// func isControlKeyPressed() -> Bool {
//   let leftCtrlDown = CGEventSource.keyState(.combinedSessionState, key: 59)
//   let rightCtrlDown = CGEventSource.keyState(.combinedSessionState, key: 62)
//   return leftCtrlDown || rightCtrlDown
// }

//

import CoreGraphics
import Foundation

let pid = ProcessInfo.processInfo.processIdentifier

print("Estate native PID: \(pid)")
let sourcePID = ProcessInfo.processInfo.processIdentifier

struct KeyboardModifierState: Codable {
  var shift: Bool = false
  var ctrl: Bool = false
  var opt: Bool = false
  var cmd: Bool = false
}
struct KeyboardModifierSnapshot: Codable {
  let leftShift: Bool
  let leftCtrl: Bool
  let leftOpt: Bool
  let leftCmd: Bool

  let rightShift: Bool
  let rightCtrl: Bool
  let rightOpt: Bool
  let rightCmd: Bool

  let fn: Bool
  let caps: Bool
}
let watchedKeyCodes: Set<CGKeyCode> = [
  58,  // Left Option
  61,  // Right Option
  59,  // Left Control
  62,  // Right Control
  63,  // Fn
  54,  // Right Command
  55,  // Left Command
  56,  // Left Shift
  60,  // Right Shift
]
let formatter = DateFormatter()
formatter.dateFormat = "HH:mm:ss.SSS"
struct ModifierState {
  var lShift = false
  var lCtrl = false
  var lOpt = false
  var lCmd = false

  var rShift = false
  var rCtrl = false
  var rOpt = false
  var rCmd = false

  var fn = false
  var caps = false
}
enum KeyDirection: String, Codable {
  case down
  case up

  var symbol: String {
    switch self {
    case .down:
      return "↓"
    case .up:
      return "↑"
    }
  }
}
enum EventSource: String, Codable {
  case cgEvent
  case hid
  case workspace
  case accessibility
}
struct ModifierSnapshot: Codable {
  let leftShift: Bool
  let leftCtrl: Bool
  let leftOpt: Bool
  let leftCmd: Bool

  let rightShift: Bool
  let rightCtrl: Bool
  let rightOpt: Bool
  let rightCmd: Bool

  let fn: Bool
  let caps: Bool

  init(from state: ModifierState) {
    self.leftShift = state.lShift
    self.leftCtrl = state.lCtrl
    self.leftOpt = state.lOpt
    self.leftCmd = state.lCmd

    self.rightShift = state.rShift
    self.rightCtrl = state.rCtrl
    self.rightOpt = state.rOpt
    self.rightCmd = state.rCmd

    self.fn = state.fn
    self.caps = state.caps
  }
}
struct CGEventInfo: Codable {
  let type: UInt32
  let keyCode: Int64?
  let flags: UInt64
  let sessionFlags: UInt64
  let sourcePID: Int64
  let sourceUserData: Int64
}
struct NativeEvent: Codable {
  enum Kind: String, Codable {
    case keyDown
    case keyUp
    case flagsChanged
    case mouse
    case hid
    case frontmostApp
  }

  let kind: Kind
  let source: EventSource
  let timestamp: UInt64
  let event: CGEventInfo?
  let name: String
  let modifiers: ModifierSnapshot?
  let direction: KeyDirection?
}
struct EventEnvelope: Codable {
  let version: UInt
  let event: NativeEvent
}
var state = ModifierState()
func modifierName(_ keyCode: Int64) -> String {
  switch keyCode {
  case 56, 60: return "SHIFT"
  case 59, 62: return "CTRL"
  case 58, 61: return "OPT"
  case 55, 54: return "CMD"
  // case 63, 105: return "FN"
  case 63: return "FN"
  case 57: return "CAPS"
  default: return "UNKNOWN"
  }
}
let modifiers: [Int64: String] = [
  56: "LSHIFT",
  60: "RSHIFT",
  59: "LCTRL",
  62: "RCTRL",
  58: "LOPT",
  61: "ROPT",
  55: "LCMD",
  54: "RCMD",
  63: "FN",
  57: "CAPS",
]
func displayKeyName(_ keyCode: Int64) -> String {
  switch keyCode {
  case 59, 62:
    return "⌃"

  case 58, 61:
    return "⌥"

  case 55, 54:
    return "⌘"

  case 56, 60:
    return "⇧"

  case 48:
    return "⇥"

  case 49:
    return "␠"

  case 36:
    return "↵"

  case 53:
    return "⎋"

  case 51:
    return "⌫"

  default:
    return keyName(keyCode)
  }
}
func keyName(_ keyCode: Int64) -> String {
  switch keyCode {
  // Letters
  case 0: return "A"
  case 1: return "S"
  case 2: return "D"
  case 3: return "F"
  case 4: return "H"
  case 5: return "G"
  case 6: return "Z"
  case 7: return "X"
  case 8: return "C"
  case 9: return "V"
  case 11: return "B"
  case 12: return "Q"
  case 13: return "W"
  case 14: return "E"
  case 15: return "R"
  case 16: return "Y"
  case 17: return "T"
  case 31: return "O"
  case 32: return "U"
  case 34: return "I"
  case 35: return "P"
  case 37: return "L"
  case 38: return "J"
  case 40: return "K"
  case 45: return "N"
  case 46: return "M"

  // Numbers
  case 18: return "1"
  case 19: return "2"
  case 20: return "3"
  case 21: return "4"
  case 23: return "5"
  case 22: return "6"
  case 26: return "7"
  case 28: return "8"
  case 25: return "9"
  case 29: return "0"

  // Whitespace / punctuation
  case 49: return "SPACE"
  case 36: return "ENTER"
  case 48: return "TAB"
  case 51: return "DELETE"
  case 53: return "ESC"
  case 41: return ";"
  case 39: return "'"
  case 43: return ","
  case 47: return "."
  case 44: return "/"
  case 42: return "\\"
  case 50: return "`"
  case 27: return "-"
  case 24: return "="
  case 33: return "["
  case 30: return "]"

  // Arrows
  case 123: return "LEFT"
  case 124: return "RIGHT"
  case 125: return "DOWN"
  case 126: return "UP"

  // Function keys
  case 122: return "F1"
  case 120: return "F2"
  case 99: return "F3"
  case 118: return "F4"
  case 96: return "F5"
  case 97: return "F6"
  case 98: return "F7"
  case 100: return "F8"
  case 101: return "F9"
  case 109: return "F10"
  case 103: return "F11"
  case 111: return "F12"

  // Modifiers
  case 56: return "LSHIFT"
  case 60: return "RSHIFT"
  case 59: return "LCTRL"
  case 62: return "RCTRL"
  case 58: return "LOPT"
  case 61: return "ROPT"
  case 55: return "LCMD"
  case 54: return "RCMD"
  case 63: return "FN"
  case 57: return "CAPS"

  default:
    return "KEY[\(keyCode)]"
  }
}
func tf(_ value: Bool) -> String {
  value ? "t" : "f"
}
func heldModifiers(_ flags: CGEventFlags) -> String {
  var result: [String] = []

  if flags.contains(.maskControl) {
    result.append("CTRL")
  }

  if flags.contains(.maskAlternate) {
    result.append("OPT")
  }

  if flags.contains(.maskCommand) {
    result.append("CMD")
  }

  if flags.contains(.maskShift) {
    result.append("SHIFT")
  }

  if flags.contains(.maskSecondaryFn) {
    result.append("FN")
  }

  return result.joined(separator: "+")
}
func eventTypeName(_ type: CGEventType) -> String {

  switch type {
  case .keyDown:
    return "KEYDOWN"

  case .keyUp:
    return "KEYUP"

  case .flagsChanged:
    return "FLAGS"

  default:
    return "\(type)"
  }
}
func eventDescription(type: CGEventType, code: Int64) -> String {
  switch type {
  case .keyDown, .keyUp:
    return keyName(code)

  case .flagsChanged:
    return modifierName(code)

  default:
    return "UNKNOWN"
  }
}
func updateModifierState(keyCode: Int64) -> KeyDirection? {
  let code = CGKeyCode(keyCode)

  switch code {
  case 56:  // Left Shift
    state.lShift.toggle()
    return state.lShift ? .down : .up

  case 60:  // Right Shift
    state.rShift.toggle()
    return state.rShift ? .down : .up

  case 59:  // Left Control
    state.lCtrl.toggle()
    return state.lCtrl ? .down : .up

  case 62:  // Right Control
    state.rCtrl.toggle()
    return state.rCtrl ? .down : .up

  case 58:  // Left Option
    state.lOpt.toggle()
    return state.lOpt ? .down : .up

  case 61:  // Right Option
    state.rOpt.toggle()
    return state.rOpt ? .down : .up

  case 55:  // Left Command
    state.lCmd.toggle()
    return state.lCmd ? .down : .up

  case 54:  // Right Command
    state.rCmd.toggle()
    return state.rCmd ? .down : .up

  case 63:  // Fn
    state.fn.toggle()
    return state.fn ? .down : .up

  case 57:  // Caps Lock
    state.caps.toggle()
    return state.caps ? .down : .up
  default:
    return nil
  }
}
func keyState(_ down: Bool, _ symbol: String) -> String {
  down ? symbol : "·"
}
func aggregateModifiers(_ modifiers: ModifierSnapshot?) -> String {
  guard let modifiers else {
    return "· · · ·"
  }

  let shift =
    modifiers.leftShift || modifiers.rightShift

  let ctrl =
    modifiers.leftCtrl || modifiers.rightCtrl

  let opt =
    modifiers.leftOpt || modifiers.rightOpt

  let cmd =
    modifiers.leftCmd || modifiers.rightCmd

  return
    "\(keyState(shift, "⇧")) "
    + "\(keyState(ctrl, "⌃")) "
    + "\(keyState(opt, "⌥")) "
    + "\(keyState(cmd, "⌘"))"
}
func printHeader() {
  print("")
  print(
    "TIME          " + "LS LC LO LM | " + "RS RC RO RM | " + "FN CP | " + "EVENT              "
      + "FLAGS"
  )

  print(
    "              " + "-- --------- | --------- | " + "-- -- | " + "------------------- "
      + "----------------"
  )

  fflush(stdout)
}
func makeEvent(
  _ event: CGEvent,
  type: CGEventType,
  description: String,
  modifierDirection: KeyDirection? = nil
) -> NativeEvent {

  let flags = event.flags
  let session = CGEventSource.flagsState(.combinedSessionState)

  let keyCode =
    event.getIntegerValueField(.keyboardEventKeycode)

  let sourcePID =
    event.getIntegerValueField(.eventSourceUnixProcessID)

  let sourceUserData =
    event.getIntegerValueField(.eventSourceUserData)

  let kind: NativeEvent.Kind

  switch type {
  case .keyDown:
    kind = .keyDown

  case .keyUp:
    kind = .keyUp

  case .flagsChanged:
    kind = .flagsChanged

  default:
    kind = .mouse
  }

  let name: String

  switch type {
  case .keyDown, .keyUp:
    name = displayKeyName(keyCode)
  default:
    name = description
  }

  let cgEventInfo = CGEventInfo(
    type: type.rawValue,
    keyCode: keyCode,
    flags: flags.rawValue,
    sessionFlags: session.rawValue,
    sourcePID: sourcePID,
    sourceUserData: sourceUserData
  )

  let modifierSnapshot = ModifierSnapshot(from: state)

  return NativeEvent(
    kind: kind,
    source: .cgEvent,
    timestamp: DispatchTime.now().uptimeNanoseconds,
    event: cgEventInfo,
    name: name,
    modifiers: modifierSnapshot,
    direction: modifierDirection
  )
}
func printEvent(_ nativeEvent: NativeEvent) {
  guard let info = nativeEvent.event else {
    return
  }

  let now = formatter.string(from: Date())
  let modifiers = nativeEvent.modifiers

  let arrow: String = {
    if let direction = nativeEvent.direction {
      return direction == .down ? "↓" : "↑"
    }

    switch nativeEvent.kind {
    case .keyDown:
      return "↓"

    case .keyUp:
      return "↑"

    default:
      return " "
    }
  }()

  let left =
    "\(keyState(modifiers?.leftShift ?? false, "⇧")) "
    + "\(keyState(modifiers?.leftCtrl ?? false, "⌃")) "
    + "\(keyState(modifiers?.leftOpt ?? false, "⌥")) "
    + "\(keyState(modifiers?.leftCmd ?? false, "⌘"))"

  let right =
    "\(keyState(modifiers?.rightShift ?? false, "⇧")) "
    + "\(keyState(modifiers?.rightCtrl ?? false, "⌃")) "
    + "\(keyState(modifiers?.rightOpt ?? false, "⌥")) "
    + "\(keyState(modifiers?.rightCmd ?? false, "⌘"))"

  let special =
    "\(keyState(modifiers?.fn ?? false, "fn")) "
    + "\(keyState(modifiers?.caps ?? false, "⇪"))"

  let flagsText = aggregateModifiers(modifiers)

  let displayName: String = {
    switch info.keyCode {
    case 55, 54:
      return "⌘"

    case 58, 61:
      return "⌥"

    case 59, 62:
      return "⌃"

    case 56, 60:
      return "⇧"

    case 63:
      return "fn"

    case 48:
      return "⇥"

    case 49:
      return "␠"

    case 36:
      return "↵"

    case 53:
      return "⎋"

    case 51:
      return "⌫"

    default:
      return nativeEvent.name
    }
  }()

  let eventDisplay = "\(arrow) \(displayName)"

  print(
    String(
      format:
        "%@ | %-9@ | %-9@ | %-4@ | %-14@ | %-8@ | %4lld | %10llu | %10llu | %6lld | %6lld",
      now as NSString,
      left as NSString,
      right as NSString,
      special as NSString,
      eventDisplay as NSString,
      flagsText as NSString,
      info.keyCode ?? -1,
      info.flags,
      info.sessionFlags,
      info.sourcePID,
      info.sourceUserData
    )
  )

  fflush(stdout)
}
let mask =
  CGEventMask(1 << CGEventType.keyDown.rawValue) | CGEventMask(1 << CGEventType.keyUp.rawValue)
  | CGEventMask(1 << CGEventType.flagsChanged.rawValue)
let callback: CGEventTapCallBack = {
  _, type, event, _ in

  // ------------------------------------------------------------
  // Event tap status
  // ------------------------------------------------------------

  if type == .tapDisabledByTimeout {
    print("!!! EVENT TAP DISABLED: TIMEOUT")
    fflush(stdout)
    return Unmanaged.passUnretained(event)
  }

  if type == .tapDisabledByUserInput {
    print("!!! EVENT TAP DISABLED: USER INPUT")
    fflush(stdout)
    return Unmanaged.passUnretained(event)
  }

  // ------------------------------------------------------------
  // Common event data
  // ------------------------------------------------------------

  let code = event.getIntegerValueField(
    .keyboardEventKeycode
  )

  // ------------------------------------------------------------
  // Event processing
  // ------------------------------------------------------------

  switch type {

  case .flagsChanged:

    // keycode 0 can appear as a flagsChanged event.
    // Do NOT interpret it as an A key press.
    if code == 0 {
      print(
        "⚠️ NON-MODIFIER FLAGS EVENT: " + "keycode=0 " + "time=\(event.timestamp) "
          + "flags=\(event.flags.rawValue)"
      )
      fflush(stdout)
      break
    }

    let name = modifierName(code)

    // IMPORTANT:
    // This is the ONLY place we mutate our reconstructed
    // modifier state for this event.
    let direction = updateModifierState(
      keyCode: code
    )

    // --------------------------------------------------------
    // Watched FLAGS diagnostic
    // --------------------------------------------------------

    if watchedKeyCodes.contains(CGKeyCode(code)) {
      let rightCommandHeld = CGEventSource.keyState(
        .combinedSessionState,
        key: 54
      )

      let leftCommandHeld = CGEventSource.keyState(
        .combinedSessionState,
        key: 55
      )

      let sourcePID = event.getIntegerValueField(
        .eventSourceUnixProcessID
      )

      let sourceState = event.getIntegerValueField(
        .eventSourceStateID
      )

      let keyboardType = event.getIntegerValueField(
        .keyboardEventKeyboardType
      )

      let autorepeat = event.getIntegerValueField(
        .keyboardEventAutorepeat
      )

      let sessionFlags =
        CGEventSource.flagsState(
          .combinedSessionState
        ).rawValue

      // print(
      //   "⚠️ WATCHED " + "\(direction?.rawValue.uppercased() ?? "?") " + "key=\(keyName(code)) "
      //     + "code=\(code) "
      //     + "time=\(event.timestamp) " + "cmd[L]=\(leftCommandHeld) "
      //     + "cmd[R]=\(rightCommandHeld) " + "flags=\(event.flags.rawValue) "
      //     + "session=\(sessionFlags) " + "pid=\(sourcePID) " + "state=\(sourceState) "
      //     + "keyboard=\(keyboardType) " + "repeat=\(autorepeat)"
      // )
      fflush(stdout)
    }
    let nativeEvent = makeEvent(
      event,
      type: type,
      description: name,
      modifierDirection: direction
    )

    printEvent(nativeEvent)
  case .keyDown:

    let name = keyName(code)

    let nativeEvent = makeEvent(
      event,
      type: type,
      description: name,
      modifierDirection: .down
    )

    printEvent(nativeEvent)

  case .keyUp:

    let name = keyName(code)

    let nativeEvent = makeEvent(
      event,
      type: type,
      description: name,
      modifierDirection: .up
    )

    printEvent(nativeEvent)

  default:
    break
  }

  return Unmanaged.passUnretained(event)
}
guard
  let tap = CGEvent.tapCreate(
    tap: .cgSessionEventTap,
    place: .headInsertEventTap,
    options: .listenOnly,
    eventsOfInterest: mask,
    callback: callback,
    userInfo: nil
  )
else {

  fputs(
    "Cannot create event tap. Check Input Monitoring permissions.\n",
    stderr
  )

  exit(1)
}
let source = CFMachPortCreateRunLoopSource(
  kCFAllocatorDefault,
  tap,
  0
)
CFRunLoopAddSource(
  CFRunLoopGetCurrent(),
  source,
  .commonModes
)
CGEvent.tapEnable(
  tap: tap,
  enable: true
)
printHeader()
print(
  "Columns: LS LC LO LM = left modifiers, " + "RS RC RO RM = right modifiers, " + "FN CP = Fn/Caps"
)
print(
  "Event flags are aggregate CoreGraphics state; "
    + "left/right state is reconstructed from keycodes."
)
print("")
fflush(stdout)
func getCurrentTimestamp() -> String {
  let formatter = DateFormatter()
  formatter.dateFormat = "HH:mm:ss.SSS"
  return formatter.string(from: Date())
}
print(
  "Columns: LS LC LO LM = left modifiers, " + "RS RC RO RM = right modifiers, " + "FN CP = Fn/Caps"
)
print(
  "Event flags are aggregate CoreGraphics state; "
    + "left/right state is reconstructed from keycodes."
)
print("")
fflush(stdout)
CFRunLoopRun()
