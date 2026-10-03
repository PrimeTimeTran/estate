import AppKit
import CoreGraphics
import Darwin
import Foundation

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
enum Kind: String, Codable {
  case keyDown = "key_down"
  case keyUp = "key_up"
  case mouseDown = "mouse_down"
  case mouseUp = "mouse_up"
  case scroll = "scroll"
  case flagsChanged = "flags_changed"
  case modifierChanged = "ModifierChanged"
}

// MARK: - Feature Flags
let enablePrintEvent = true
let enableEstateSocket = true
let enableWorkspaceObserver = true
let enableInitialForegroundApp = true
let enableCGEventTap = true

let enableEventLogging = true
let enableWorkspaceLogging = true
var estateClientFD: Int32 = -1
var state = ModifierState()

// MARK: - Shared Observer State
formatter.dateFormat = "HH:mm:ss.SSS"

var shouldStartEstateSocket = true
let socketPath = "/tmp/estate-hid.sock"
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
let watchedKeyCodes: Set<CGKeyCode> = [
  57,  // CAPS
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
let pid = ProcessInfo.processInfo.processIdentifier
let sourcePID = ProcessInfo.processInfo.processIdentifier
let formatter = DateFormatter()
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

struct FrontmostApp: Codable {
  let bundleID: String?
  let name: String?
  let pid: Int32?
}
struct EstateNativeEventMessage: Codable {
  let type: String
  let event: NativeEvent

  init(event: NativeEvent) {
    self.type = "native_event"
    self.event = event
  }
}
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
  let sentAt: UInt64
  let kind: Kind
  let source: EventSource
  let timestamp: UInt64

  let keyCode: UInt16?

  let button: Int64?
  let x: Double?
  let y: Double?

  let vertical: Int64?
  let horizontal: Int64?

  let name: String
  let modifiers: ModifierSnapshot?
  let direction: KeyDirection?

  let frontmostApp: FrontmostApp?

  enum Kind: String, Codable {
    case keyDown = "key_down"
    case keyUp = "key_up"
    case mouseDown = "mouse_down"
    case mouseUp = "mouse_up"
    case scroll = "scroll"
    case flagsChanged = "flags_changed"
    case frontmostApp = "frontmost_app"
    case modifierChanged = "ModifierChanged"
  }

  enum CodingKeys: String, CodingKey {
    case sentAt = "sent_at"
    case kind
    case source
    case timestamp
    case keyCode = "key_code"
    case button
    case x
    case y
    case vertical
    case horizontal
    case name
    case modifiers
    case direction
    case frontmostApp
  }
}
struct MouseButton: Codable {
  let number: Int64
}

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
func updateModifierState(
  keyCode: Int64,
  flags: CGEventFlags
) -> KeyDirection? {
  let code = CGKeyCode(keyCode)

  switch code {
  case 56:  // Left Shift
    let down = flags.contains(.maskShift)
    state.lShift = down
    return down ? .down : .up

  case 60:  // Right Shift
    let down = flags.contains(.maskShift)
    state.rShift = down
    return down ? .down : .up

  case 59:  // Left Control
    let down = flags.contains(.maskControl)
    state.lCtrl = down
    return down ? .down : .up

  case 62:  // Right Control
    let down = flags.contains(.maskControl)
    state.rCtrl = down
    return down ? .down : .up

  case 58:  // Left Option
    let down = flags.contains(.maskAlternate)
    state.lOpt = down
    return down ? .down : .up

  case 61:  // Right Option
    let down = flags.contains(.maskAlternate)
    state.rOpt = down
    return down ? .down : .up

  case 55:  // Left Command
    let down = flags.contains(.maskCommand)
    state.lCmd = down
    return down ? .down : .up

  case 54:  // Right Command
    let down = flags.contains(.maskCommand)
    state.rCmd = down
    return down ? .down : .up

  case 63:  // Fn
    let down = flags.contains(.maskSecondaryFn)
    state.fn = down
    return down ? .down : .up

  case 57:  // Caps Lock
    let down = flags.contains(.maskAlphaShift)
    state.caps = down
    return down ? .down : .up

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

func printEvent(_ nativeEvent: NativeEvent) {
  let now = formatter.string(from: Date())
  let modifiers = nativeEvent.modifiers

  let arrow: String = {
    if let direction = nativeEvent.direction {
      return direction == .down ? "↓" : "↑"
    }

    switch nativeEvent.kind {
    case .keyDown, .mouseDown:
      return "↓"

    case .keyUp, .mouseUp:
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

  // Semantic key code first. Fall back to the raw CGEvent
  // for debugging if it still exists.
  let keyCode: Int64? = {
    guard let keyCode = nativeEvent.keyCode else {
      return nil
    }

    return Int64(keyCode)
  }()

  let displayName: String = {
    guard let keyCode else {
      return nativeEvent.name
    }

    switch keyCode {
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
        "%@ | %-9@ | %-9@ | %-4@ | %-14@ | %-8@ | %4lld",
      now as NSString,
      left as NSString,
      right as NSString,
      special as NSString,
      eventDisplay as NSString,
      flagsText as NSString,
      keyCode ?? -1
    )
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
  let now = DispatchTime.now().uptimeNanoseconds
  switch type {
  case .keyDown:
    return NativeEvent(
      sentAt: now,
      kind: .keyDown,
      source: .cgEvent,
      timestamp: DispatchTime.now().uptimeNanoseconds,
      keyCode: UInt16(keyCode),
      button: nil,
      x: nil,
      y: nil,
      vertical: nil,
      horizontal: nil,
      // event: CGEventInfo(
      //   type: type.rawValue,
      //   keyCode: keyCode,
      //   flags: flags.rawValue,
      //   sessionFlags: session.rawValue,
      //   sourcePID: sourcePID,
      //   sourceUserData: sourceUserData
      // ),
      name: keyName(keyCode),
      modifiers: ModifierSnapshot(from: state),
      direction: .down,
      frontmostApp: nil
    )

  case .keyUp:
    return NativeEvent(
      sentAt: now,
      kind: .keyUp,
      source: .cgEvent,
      timestamp: DispatchTime.now().uptimeNanoseconds,
      keyCode: UInt16(keyCode),
      button: nil,
      x: nil,
      y: nil,
      vertical: nil,
      horizontal: nil,
      // event: CGEventInfo(
      //   type: type.rawValue,
      //   keyCode: keyCode,
      //   flags: flags.rawValue,
      //   sessionFlags: session.rawValue,
      //   sourcePID: sourcePID,
      //   sourceUserData: sourceUserData
      // ),
      name: keyName(keyCode),
      modifiers: ModifierSnapshot(from: state),
      direction: .up,
      frontmostApp: nil
    )

  case .flagsChanged:
    return NativeEvent(
      sentAt: now,
      kind: .flagsChanged,
      source: .cgEvent,
      timestamp: now,

      keyCode: UInt16(keyCode),

      button: nil,
      x: nil,
      y: nil,
      vertical: nil,
      horizontal: nil,

      name: description,
      modifiers: ModifierSnapshot(from: state),
      direction: modifierDirection,
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

    let kind: NativeEvent.Kind =
      switch type {
      case .leftMouseDown,
        .rightMouseDown,
        .otherMouseDown:
        .mouseDown

      case .leftMouseUp,
        .rightMouseUp,
        .otherMouseUp:
        .mouseUp

      default:
        fatalError("Unexpected mouse event type")
      }

    let location = event.location

    return NativeEvent(
      sentAt: now,
      kind: kind,
      source: .cgEvent,
      timestamp: DispatchTime.now().uptimeNanoseconds,

      keyCode: nil,
      button: buttonNumber,
      x: location.x,
      y: location.y,
      vertical: nil,
      horizontal: nil,

      name: "button \(buttonNumber)",
      modifiers: ModifierSnapshot(from: state),
      direction: direction,
      frontmostApp: nil
    )

  default:
    fatalError(
      "Unsupported CGEventType: \(type.rawValue)"
    )
  }
}
func sendNativeEvent(
  _ event: NativeEvent,
  on clientFD: Int32
) {
  guard clientFD >= 0 else {
    if enablePrintEvent {
      // print("⚠️ SWIFT → RUST skipped: invalid client fd=\(clientFD)")
      printEvent(event)
    }
    return
  }
  do {
    let message = EstateNativeEventMessage(event: event)
    let data = try JSONEncoder().encode(message)

    if enablePrintEvent {
      printEvent(event)
    }

    guard let json = String(data: data, encoding: .utf8) else {
      return
    }

    sendEstate(json, on: clientFD)
  } catch {
    print("❌ failed to encode native event: \(error)")
  }
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

func emitForegroundApp(_ app: FrontmostApp) {
  let now = DispatchTime.now().uptimeNanoseconds

  let event = NativeEvent(
    sentAt: now,
    kind: .frontmostApp,
    source: .workspace,
    timestamp: mach_absolute_time(),

    keyCode: nil,
    button: nil,
    x: nil,
    y: nil,
    vertical: nil,
    horizontal: nil,

    name: app.name ?? "",
    modifiers: nil,
    direction: nil,
    frontmostApp: app
  )

  sendNativeEvent(
    event,
    on: estateClientFD
  )
}
func startEstateSocket() {
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
  while true {
    let clientFD = accept(
      serverFD,
      nil,
      nil
    )
    guard clientFD >= 0 else {
      print(
        "accept failed: "
          + "\(String(cString: strerror(errno)))"
      )
      continue
    }

    estateClientFD = clientFD

    print(
      "🔥 ESTATE RUST CONNECTED fd=\(clientFD)"
    )

    handleEstateConnection(clientFD)

    close(clientFD)

    estateClientFD = -1
    print("🔌 ESTATE RUST DISCONNECTED")

    handleEstateConnection(
      estateClientFD
    )
    // startSwiftPingLoop(clientFD)
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
func machNow() -> UInt64 {
  mach_absolute_time()
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

// MARK: - Estate Transport
if enableEstateSocket {
  print("🚀 Starting Estate socket...")

  Thread {
    startEstateSocket()
  }.start()
} else {
  print("⏭️ Estate socket disabled")
}

// MARK: - Initial Foreground Application
if enableInitialForegroundApp {
  let initialApp = frontmostApplication()

  print("🚀 Initial frontmost app:")
  print("   name: \(initialApp.name ?? "nil")")
  print("   bundle: \(initialApp.bundleID ?? "nil")")
  print("   pid: \(initialApp.pid.map(String.init) ?? "nil")")

  if enableEstateSocket {
    emitForegroundApp(initialApp)
  }
}

// MARK: - Workspace / Foreground Application Observer
if enableWorkspaceObserver {
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
      if enableWorkspaceLogging {
        // print("⚠️ activation notification without app")
      }
      return
    }

    let frontmost = FrontmostApp(
      bundleID: app.bundleIdentifier,
      name: app.localizedName,
      pid: app.processIdentifier
    )

    if enableWorkspaceLogging {
      print("")
      print("🔥 FOREGROUND APP CHANGED")
      print("   name: \(frontmost.name ?? "nil")")
      print("   bundle: \(frontmost.bundleID ?? "nil")")
      print("   pid: \(frontmost.pid.map(String.init) ?? "nil")")
    }

    if enableEstateSocket {
      emitForegroundApp(frontmost)
    }
  }

  print("")
  print("👀 Watching for foreground application changes...")
  print("   Try ⌘Tab between applications.")
} else {
  print("⏭️ Workspace observer disabled")
}

// MARK: - Keyboard / CGEvent Observation
if enableCGEventTap {
  formatter.dateFormat = "HH:mm:ss.SSS"

  let callback: CGEventTapCallBack = {
    proxy,
    type,
    event,
    userInfo in
    // print("🔥 CGEVENT TYPE: \(type.rawValue)")
    let code = event.getIntegerValueField(
      .keyboardEventKeycode
    )

    if type == .flagsChanged {
      print(
        "🔥 RAW FLAGS: keyCode=\(code) " + "flags=0x\(String(event.flags.rawValue, radix: 16))"
      )
      fflush(stdout)
    }
    fflush(stdout)

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
    switch type {
    // MARK: Modifier

    case .flagsChanged:
      // Only normalize actual modifier keys.
      guard watchedKeyCodes.contains(CGKeyCode(code)) else {
        if enableEventLogging {
          print(
            "⚠️ IGNORED FLAGS EVENT: "
              + "keycode=\(code) "
              + "time=\(event.timestamp) "
              + "flags=0x\(String(event.flags.rawValue, radix: 16))"
          )
          fflush(stdout)
        }

        break
      }
      let name = modifierName(code)
      let direction = updateModifierState(keyCode: code, flags: event.flags)

      print(
        """
        🧠 STATE AFTER FLAGS
           keyCode=\(code)
           flags=0x\(String(event.flags.rawValue, radix: 16))
           L: shift=\(state.lShift) ctrl=\(state.lCtrl) opt=\(state.lOpt) cmd=\(state.lCmd)
           R: shift=\(state.rShift) ctrl=\(state.rCtrl) opt=\(state.rOpt) cmd=\(state.rCmd)
           fn=\(state.fn) caps=\(state.caps)
        """
      )
      fflush(stdout)

      let nativeEvent = makeEvent(
        event,
        type: type,
        description: name,
        modifierDirection: direction
      )

      if enableEstateSocket {
        sendNativeEvent(
          nativeEvent,
          on: estateClientFD
        )
      }
    // 54  right command
    // 55  left command
    // 56  left shift
    // 57  caps lock
    // 58  left option
    // 59  left control
    // 60  right shift
    // 61  right option
    // 62  right control
    // 63  fn
    // MARK: Key Down
    case .keyDown:

      let name = keyName(code)

      let nativeEvent = makeEvent(
        event,
        type: type,
        description: name,
        modifierDirection: .down
      )

      if enableEstateSocket {
        sendNativeEvent(
          nativeEvent,
          on: estateClientFD
        )
      }

    // MARK: Key Up

    case .keyUp:

      let name = keyName(code)

      let nativeEvent = makeEvent(
        event,
        type: type,
        description: name,
        modifierDirection: .up
      )

      if enableEstateSocket {
        sendNativeEvent(
          nativeEvent,
          on: estateClientFD
        )
      }

    // MARK: Mouse

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

      let kind: NativeEvent.Kind =
        switch type {
        case .leftMouseDown,
          .rightMouseDown,
          .otherMouseDown:
          .mouseDown

        case .leftMouseUp,
          .rightMouseUp,
          .otherMouseUp:
          .mouseUp

        default:
          fatalError("Unexpected mouse event type")
        }

      let location = event.location

      let nativeEvent = NativeEvent(
        sentAt: DispatchTime.now().uptimeNanoseconds,
        kind: kind,
        source: .cgEvent,
        timestamp: event.timestamp,

        keyCode: nil,

        button: buttonNumber,
        x: location.x,
        y: location.y,

        vertical: nil,
        horizontal: nil,

        name: "Mouse \(buttonNumber + 1)",
        modifiers: ModifierSnapshot(from: state),
        direction: direction,
        frontmostApp: nil
      )

      if enableEstateSocket {
        sendNativeEvent(
          nativeEvent,
          on: estateClientFD
        )
      }

    default:
      break
    }

    return Unmanaged.passUnretained(event)
  }

  // MARK: Install Event Tap

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

  print("👀 Watching CGEvents...")

  printHeader()

  print(
    "Columns: LS LC LO LM = left modifiers, "
      + "RS RC RO RM = right modifiers, "
      + "FN CP = Fn/Caps"
  )

  print(
    "Event flags are aggregate CoreGraphics state; "
      + "left/right state is reconstructed from keycodes."
  )

  print("")
  fflush(stdout)

} else {
  print("⏭️ CGEvent tap disabled")
}

// MARK: - Main Run Loop
print("")
print("🚀 Estate native PID: \(pid)")
print("")

CFRunLoopRun()
