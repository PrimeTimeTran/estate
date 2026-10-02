import AppKit
import CoreGraphics
import Darwin
import Foundation

struct FrontmostApp: Codable {
  let bundleID: String?
  let name: String?
  let pid: Int32?
}
struct EventEnvelope: Codable {
  let type: String
  let version: UInt
  let event: NativeEvent
}

func emitForegroundApp(_ app: FrontmostApp) {
  let event = NativeEvent(
    kind: .frontmostApp,
    source: .workspace,
    timestamp: mach_absolute_time(),
    event: nil,
    name: app.name ?? "",
    modifiers: nil,
    direction: nil,
    mouseButton: nil,
    clickCount: nil,
    locationX: nil,
    locationY: nil,
    frontmostApp: app
  )

  let envelope = EventEnvelope(
    type: "native_event",
    version: 1,
    event: event
  )

  do {
    let data = try JSONEncoder().encode(envelope)

    if let json = String(data: data, encoding: .utf8) {
      print("🍎 \(json)")
    }
  } catch {
    print("❌ failed to encode event: \(error)")
  }
}
struct EstateNativeEventMessage: Codable {
  let type: String
  let event: NativeEvent

  init(event: NativeEvent) {
    self.type = "native_event"
    self.event = event
  }
}
func frontmostApplication() -> FrontmostApp {
  let app = NSWorkspace.shared.frontmostApplication
  return FrontmostApp(
    bundleID: app?.bundleIdentifier,
    name: app?.localizedName,
    pid: app?.processIdentifier
  )
}

func emitFrontmostApp(_ app: FrontmostApp) {
  let nativeEvent = NativeEvent(
    kind: .frontmostApp,
    source: .workspace,
    timestamp: mach_absolute_time(),

    event: nil,
    name: app.name ?? "",
    modifiers: nil,
    direction: nil,

    mouseButton: nil,
    clickCount: nil,
    locationX: nil,
    locationY: nil,

    frontmostApp: app
  )

  let envelope = EventEnvelope(
    type: "native_event",
    version: 1,
    event: nativeEvent
  )

  do {
    let data = try JSONEncoder().encode(envelope)

    guard
      let message = String(
        data: data,
        encoding: .utf8
      )
    else {
      return
    }

    guard estateClientFD >= 0 else {
      return
    }

    sendEstate(
      message,
      on: estateClientFD
    )
  } catch {
    print(
      "❌ failed to encode frontmost app: \(error)"
    )
  }
}

// MARK: - Initial state

let initialApp = frontmostApplication()

print("🚀 Initial frontmost app:")
print("   name: \(initialApp.name ?? "nil")")
print("   bundle: \(initialApp.bundleID ?? "nil")")
print("   pid: \(initialApp.pid.map(String.init) ?? "nil")")

// MARK: - App activation events

NSWorkspace.shared.notificationCenter.addObserver(
  forName: NSWorkspace.didActivateApplicationNotification,
  object: nil,
  queue: .main
) { notification in
  guard
    let app =
      notification.userInfo?[
        NSWorkspace.applicationUserInfoKey
      ] as? NSRunningApplication
  else {
    print("⚠️ activation notification without app")
    return
  }

  let frontmost = FrontmostApp(
    bundleID: app.bundleIdentifier,
    name: app.localizedName,
    pid: app.processIdentifier
  )

  print("")
  print("🔥 FOREGROUND APP CHANGED")
  print("   name: \(frontmost.name ?? "nil")")
  print("   bundle: \(frontmost.bundleID ?? "nil")")
  print("   pid: \(frontmost.pid.map(String.init) ?? "nil")")

  emitForegroundApp(frontmost)
}

// MARK: - Run loop

print("")
print("👀 Watching for foreground application changes...")
print("   Try ⌘Tab between applications.")
print("")

var estateClientFD: Int32 = -1
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
struct ModifierState: Codable {
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

  enum CodingKeys: String, CodingKey {
    case lShift = "shift_left"
    case rShift = "shift_right"

    case lCtrl = "ctrl_left"
    case rCtrl = "ctrl_right"

    case lOpt = "opt_left"
    case rOpt = "opt_right"

    case lCmd = "cmd_left"
    case rCmd = "cmd_right"

    case fn = "function"
    case caps = "caps"
  }
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

  enum CodingKeys: String, CodingKey {
    case leftShift = "shift_left"
    case leftCtrl = "ctrl_left"
    case leftOpt = "opt_left"
    case leftCmd = "cmd_left"

    case rightShift = "shift_right"
    case rightCtrl = "ctrl_right"
    case rightOpt = "opt_right"
    case rightCmd = "cmd_right"

    case fn = "function"
    case caps = "caps"
  }

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

  let mouseButton: Int64?
  let clickCount: Int64?
  let locationX: Double?
  let locationY: Double?

  let frontmostApp: FrontmostApp?
}
enum NativeEventKind: String, Codable {
  case keyDown
  case keyUp
  case flagsChanged
  case mouse
  case hid
  case frontmostApp
}
struct EstateNativeEvent: Codable {
  let sentAt: UInt64
  let kind: NativeEventKind
  let modifiers: ModifierSnapshot
  let source: EventSource
  let timestamp: UInt64
  let name: String
  let direction: KeyDirection?
  let event: CGEventInfo?

  enum CodingKeys: String, CodingKey {
    case sentAt = "sent_at"
    case kind
    case modifiers
    case source
    case timestamp
    case name
    case direction
    case event
  }
}
enum CodingKeys: String, CodingKey {
  case sentAt = "sent_at"
  case kind
  case modifiers
  case source
  case timestamp
  case name
  case direction
  case event
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
func sendNativeEvent(
  _ event: NativeEvent,
  on clientFD: Int32
) {
  do {
    let message = EstateNativeEventMessage(event: event)
    let data = try JSONEncoder().encode(message)

    guard let json = String(data: data, encoding: .utf8) else {
      return
    }

    sendEstate(json, on: clientFD)
  } catch {
    print("❌ failed to encode native event: \(error)")
  }
}
func makeEvent(
  _ event: CGEvent,
  type: CGEventType,
  description: String,
  modifierDirection: KeyDirection? = nil
) -> NativeEvent {

  let flags = event.flags
  let session =
    CGEventSource.flagsState(.combinedSessionState)

  let keyCode =
    event.getIntegerValueField(
      .keyboardEventKeycode
    )

  let sourcePID =
    event.getIntegerValueField(
      .eventSourceUnixProcessID
    )

  let sourceUserData =
    event.getIntegerValueField(
      .eventSourceUserData
    )

  switch type {

  case .keyDown:
    return NativeEvent(
      kind: .keyDown,
      source: .cgEvent,
      timestamp: DispatchTime.now().uptimeNanoseconds,
      event: CGEventInfo(
        type: type.rawValue,
        keyCode: keyCode,
        flags: flags.rawValue,
        sessionFlags: session.rawValue,
        sourcePID: sourcePID,
        sourceUserData: sourceUserData
      ),
      name: keyName(keyCode),
      modifiers: ModifierSnapshot(from: state),
      direction: .down,
      mouseButton: nil,
      clickCount: nil,
      locationX: nil,
      locationY: nil,
      frontmostApp: nil
    )

  case .keyUp:
    return NativeEvent(
      kind: .keyUp,
      source: .cgEvent,
      timestamp: DispatchTime.now().uptimeNanoseconds,
      event: CGEventInfo(
        type: type.rawValue,
        keyCode: keyCode,
        flags: flags.rawValue,
        sessionFlags: session.rawValue,
        sourcePID: sourcePID,
        sourceUserData: sourceUserData
      ),
      name: keyName(keyCode),
      modifiers: ModifierSnapshot(from: state),
      direction: .up,
      mouseButton: nil,
      clickCount: nil,
      locationX: nil,
      locationY: nil,
      frontmostApp: nil
    )

  case .flagsChanged:
    return NativeEvent(
      kind: .flagsChanged,
      source: .cgEvent,
      timestamp: DispatchTime.now().uptimeNanoseconds,
      event: CGEventInfo(
        type: type.rawValue,
        keyCode: keyCode,
        flags: flags.rawValue,
        sessionFlags: session.rawValue,
        sourcePID: sourcePID,
        sourceUserData: sourceUserData
      ),
      name: description,
      modifiers: ModifierSnapshot(from: state),
      direction: modifierDirection,
      mouseButton: nil,
      clickCount: nil,
      locationX: nil,
      locationY: nil,
      frontmostApp: nil
    )

  case .leftMouseDown,
    .leftMouseUp,
    .rightMouseDown,
    .rightMouseUp,
    .otherMouseDown,
    .otherMouseUp:

    let buttonNumber =
      event.getIntegerValueField(
        .mouseEventButtonNumber
      )

    let direction: KeyDirection =
      switch type {
      case .leftMouseDown,
        .rightMouseDown,
        .otherMouseDown:
        .down

      case .leftMouseUp,
        .rightMouseUp,
        .otherMouseUp:
        .up

      default:
        fatalError("Unexpected mouse event type")
      }

    let clickCount =
      event.getIntegerValueField(
        .mouseEventClickState
      )

    let location = event.location

    return NativeEvent(
      kind: .mouse,
      source: .cgEvent,
      timestamp: DispatchTime.now().uptimeNanoseconds,
      event: CGEventInfo(
        type: type.rawValue,
        keyCode: nil,
        flags: flags.rawValue,
        sessionFlags: session.rawValue,
        sourcePID: sourcePID,
        sourceUserData: sourceUserData
      ),
      name: "button \(buttonNumber)",
      modifiers: ModifierSnapshot(from: state),
      direction: direction,
      mouseButton: buttonNumber,
      clickCount: clickCount,
      locationX: location.x,
      locationY: location.y,
      frontmostApp: nil
    )

  default:
    fatalError(
      "Unsupported CGEventType: \(type.rawValue)"
    )
  }
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

  // print(
  //   String(
  //     format:
  //       "%@ | %-9@ | %-9@ | %-4@ | %-14@ | %-8@ | %4lld | %10llu | %10llu | %6lld | %6lld",
  //     now as NSString,
  //     left as NSString,
  //     right as NSString,
  //     special as NSString,
  //     eventDisplay as NSString,
  //     flagsText as NSString,
  //     info.keyCode ?? -1,
  //     info.flags,
  //     info.sessionFlags,
  //     info.sourcePID,
  //     info.sourceUserData
  //   )
  // )
  //   let sentAt = mach_absolute_time()
  //
  //   let message =
  //     "{\"type\":\"native_event\"," + "\"event\":{" + "\"sent_at\":\(sentAt),"
  //     + "\"kind\":\"key_down\"," + "\"key_code\":\(displayName)" + "}}"
  //   guard estateClientFD >= 0 else {
  //     return
  //   }
  //
  //   sendEstate(
  //     message,
  //     on: estateClientFD
  //   )
  // sendEstate(
  //   message,
  //   on: clientFD
  // )
  // sendEstate(
  //   message,
  //   on: estateClientFD
  // )

  fflush(stdout)
}
let mask =
  CGEventMask(1 << CGEventType.keyDown.rawValue)
  | CGEventMask(1 << CGEventType.keyUp.rawValue)
  | CGEventMask(1 << CGEventType.flagsChanged.rawValue)
  | CGEventMask(1 << CGEventType.leftMouseDown.rawValue)
  | CGEventMask(1 << CGEventType.leftMouseUp.rawValue)
  | CGEventMask(1 << CGEventType.rightMouseDown.rawValue)
  | CGEventMask(1 << CGEventType.rightMouseUp.rawValue)
  | CGEventMask(1 << CGEventType.otherMouseDown.rawValue)
  | CGEventMask(1 << CGEventType.otherMouseUp.rawValue)
  | CGEventMask(1 << CGEventType.scrollWheel.rawValue)

let callback: CGEventTapCallBack = {
  proxy,
  type,
  event,
  userInfo in

  let rawKeyCode =
    event.getIntegerValueField(
      .keyboardEventKeycode
    )

  let mouseButton =
    event.getIntegerValueField(
      .mouseEventButtonNumber
    )

  let location = event.location

  let rawLabel: String

  let sentAt = mach_absolute_time()

  switch type {
  case .keyDown:
    let message =
      "{\"type\":\"native_event\"," + "\"event\":{" + "\"sent_at\":\(sentAt),"
      + "\"kind\":\"key_down\"," + "\"key_code\":\(rawKeyCode)" + "}}"

    guard estateClientFD >= 0 else {
      return Unmanaged.passUnretained(event)
    }

    sendEstate(
      message,
      on: estateClientFD
    )
  case .keyUp:
    let message =
      "{\"type\":\"native_event\"," + "\"event\":{" + "\"sent_at\":\(sentAt),"
      + "\"kind\":\"key_up\"," + "\"key_code\":\(rawKeyCode)" + "}}"

    guard estateClientFD >= 0 else {
      return Unmanaged.passUnretained(event)
    }

    sendEstate(
      message,
      on: estateClientFD
    )
  case .leftMouseDown,
    .rightMouseDown,
    .otherMouseDown:

    let message =
      "{\"type\":\"native_event\"," + "\"event\":{" + "\"sent_at\":\(sentAt),"
      + "\"kind\":\"mouse_down\"," + "\"button\":\(mouseButton)," + "\"x\":\(location.x),"
      + "\"y\":\(location.y)" + "}}"

    guard estateClientFD >= 0 else {
      return Unmanaged.passUnretained(event)
    }

    sendEstate(
      message,
      on: estateClientFD
    )

  case .leftMouseUp,
    .rightMouseUp,
    .otherMouseUp:

    let message =
      "{\"type\":\"native_event\"," + "\"event\":{" + "\"sent_at\":\(sentAt),"
      + "\"kind\":\"mouse_up\"," + "\"button\":\(mouseButton)," + "\"x\":\(location.x),"
      + "\"y\":\(location.y)" + "}}"

    guard estateClientFD >= 0 else {
      return Unmanaged.passUnretained(event)
    }

    sendEstate(
      message,
      on: estateClientFD
    )
  case .scrollWheel:

    let vertical =
      event.getIntegerValueField(
        .scrollWheelEventDeltaAxis1
      )

    let horizontal =
      event.getIntegerValueField(
        .scrollWheelEventDeltaAxis2
      )

    let message =
      "{\"type\":\"native_event\"," + "\"event\":{" + "\"sent_at\":\(sentAt),"
      + "\"kind\":\"scroll\"," + "\"vertical\":\(vertical)," + "\"horizontal\":\(horizontal)" + "}}"

    guard estateClientFD >= 0 else {
      return Unmanaged.passUnretained(event)
    }

    sendEstate(
      message,
      on: estateClientFD
    )

  case .flagsChanged:
    break
  // Don't send yet.

  default:
    break
  }
  // print(
  //   "RAW \(rawLabel) " + "type=\(type.rawValue) " + "button=\(mouseButton) "
  //     + "keyCode=\(rawKeyCode)"
  // )

  fflush(stdout)
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
    sendNativeEvent(nativeEvent, on: estateClientFD)
  case .keyDown:
    let name = keyName(code)
    let nativeEvent = makeEvent(
      event,
      type: type,
      description: name,
      modifierDirection: .down
    )
    sendNativeEvent(nativeEvent, on: estateClientFD)
  case .keyUp:
    let name = keyName(code)
    let nativeEvent = makeEvent(
      event,
      type: type,
      description: name,
      modifierDirection: .up
    )
    sendNativeEvent(nativeEvent, on: estateClientFD)
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


func sendEstate(
  _ message: String,
  on clientFD: Int32
) {
  let payload = message + "\n"
// 
//   print("💜 SEND ATTEMPT fd=\(clientFD) bytes=\(payload.utf8.count)")
//   print("💜 SEND DATA \(payload.trimmingCharacters(in: .newlines))")

  payload.withCString { ptr in
    let length = strlen(ptr)

    let result = write(
      clientFD,
      ptr,
      length
    )

    if result < 0 {
      print(
        "❌ SWIFT → RUST write failed: \(String(cString: strerror(errno)))"
      )
    } else {
      // print(
      //   "💜 SWIFT → RUST SENT fd=\(clientFD) bytes=\(result)/\(length)"
      // )
    }
  }
}

func sendKeyEvent(
  kind: String,
  keyCode: Int64,
  sentAt: Int64
) {
  let message: [String: Any] = [
    "type": "native_event",
    "event": [
      "sent_at": sentAt,
      "kind": kind,
      "key_code": keyCode,
    ],
  ]
  do {
    let data = try JSONSerialization.data(
      withJSONObject: message,
      options: [.sortedKeys]
    )
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write(Data([0x0A]))
  } catch {
    fputs("JSON serialization error: \(error)\n", stderr)
  }
}
func startSwiftPingLoop(
  _ clientFD: Int32
) {
  Thread {
    var id: UInt64 = 0

    while true {
      sleep(10)

      id += 1

      let message =
        #"{"type":"ping","id":"#
        + "\(id)"
        + "}"

      guard estateClientFD >= 0 else {
        continue
      }

      sendEstate(
        message,
        on: estateClientFD
      )
    }
  }.start()
}
func handleEstateConnection(
  _ clientFD: Int32
) {
  while true {
    var buffer = [UInt8](
      repeating: 0,
      count: 4096
    )

    let count = read(
      clientFD,
      &buffer,
      buffer.count
    )

    if count <= 0 {
      print(
        "Rust disconnected fd=\(clientFD)"
      )

      if estateClientFD == clientFD {
        estateClientFD = -1
      }

      close(clientFD)
      return
    }

    let message =
      String(
        bytes: buffer[..<count],
        encoding: .utf8
      ) ?? ""

    print(
      "RUST → SWIFT: "
        + message.trimmingCharacters(
          in: .whitespacesAndNewlines
        )
    )

    if message.contains("\"ping\"") {
      let response =
        #"{"type":"pong","id":1}"# + "\n"

      response.withCString { ptr in
        _ = write(
          clientFD,
          ptr,
          strlen(ptr)
        )
      }

      print(
        " SWIFT → RUST: PONG"
      )
    }
  }
}
func startEstateSocket() {
  let socketPath = "/tmp/estate-hid.sock"

  try? FileManager.default.removeItem(
    atPath: socketPath
  )

  let serverFD = socket(
    AF_UNIX,
    SOCK_STREAM,
    0
  )

  guard serverFD >= 0 else {
    fatalError("failed to create socket")
  }

  var address = sockaddr_un()
  address.sun_family = sa_family_t(AF_UNIX)

  withUnsafeMutableBytes(
    of: &address.sun_path
  ) { buffer in
    let bytes =
      socketPath.utf8CString.map {
        UInt8(bitPattern: $0)
      }

    buffer.copyBytes(from: bytes)
  }

  let bindResult =
    withUnsafePointer(to: &address) {
      $0.withMemoryRebound(
        to: sockaddr.self,
        capacity: 1
      ) {
        bind(
          serverFD,
          $0,
          socklen_t(
            MemoryLayout<sockaddr_un>.size
          )
        )
      }
    }

  guard bindResult == 0 else {
    fatalError(
      "failed to bind \(socketPath): " + "\(String(cString: strerror(errno)))"
    )
  }

  guard listen(serverFD, 1) == 0 else {
    fatalError(
      "failed to listen: " + "\(String(cString: strerror(errno)))"
    )
  }

  // print(
  //   "ESTATE HID listening: \(socketPath)"
  // )

  while true {
    let clientFD = accept(
      serverFD,
      nil,
      nil
    )

    guard clientFD >= 0 else {
      print(
        "accept failed: " + "\(String(cString: strerror(errno)))"
      )
      continue
    }

    estateClientFD = clientFD

    print(
      "ESTATE RUST connected fd=\(estateClientFD)"
    )

    // startSwiftPingLoop(clientFD)

    handleEstateConnection(
      estateClientFD
    )
  }
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
Thread {
  startEstateSocket()
}.start()
CFRunLoopRun()

struct MouseButton: Codable {
  let number: Int64
}

let pid = ProcessInfo.processInfo.processIdentifier

print("Estate native PID: \(pid)")
let sourcePID = ProcessInfo.processInfo.processIdentifier

let socketPath = "/tmp/estate-hid.sock"

try? FileManager.default.removeItem(atPath: socketPath)

let serverFD = socket(AF_UNIX, SOCK_STREAM, 0)

guard serverFD >= 0 else {
  fatalError("failed to create socket")
}

var address = sockaddr_un()
address.sun_family = sa_family_t(AF_UNIX)

withUnsafeMutableBytes(of: &address.sun_path) { buffer in
  let bytes = socketPath.utf8CString.map { UInt8(bitPattern: $0) }
  buffer.copyBytes(from: bytes)
}
let bindResult = withUnsafePointer(to: &address) {
  $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
    bind(serverFD, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
  }
}

guard bindResult == 0 else {
  fatalError("failed to bind \(socketPath)")
}

guard listen(serverFD, 1) == 0 else {
  fatalError("failed to listen")
}

// print("ESTATE HID listening: \(socketPath)")

let clientFD = accept(serverFD, nil, nil)

guard clientFD >= 0 else {
  fatalError("failed to accept Rust connection")
}

print("ESTATE RUST connected")

while true {
  var buffer = [UInt8](repeating: 0, count: 4096)

  let count = read(
    clientFD,
    &buffer,
    buffer.count
  )

  if count <= 0 {
    print("Rust disconnected")
    break
  }

  let message =
    String(
      bytes: buffer[..<count],
      encoding: .utf8
    ) ?? ""

  print("RUST → SWIFT: \(message.trimmingCharacters(in: .whitespacesAndNewlines))")

  if message.contains("PING") {
    let response = "PONG\n"

    response.withCString { ptr in
      _ = write(
        clientFD,
        ptr,
        strlen(ptr)
      )
    }
    print("SWIFT → RUST: PONG")
  }
}

emitForegroundApp(initialApp)
func machNow() -> UInt64 {
  mach_absolute_time()
}
