import CoreGraphics
import Foundation

final class EventSender {
    private let handle: FileHandle

    init(path: String) throws {
        // Unix socket connection goes here.
        //
        // In production I'd wrap the POSIX socket APIs rather
        // than repeatedly opening files.
        fatalError("socket implementation")
    }

    func send(_ event: String) throws {
        let data = (event + "\n").data(using: .utf8)!
        try handle.write(contentsOf: data)
    }
}

let watchedKeyCodes: Set<CGKeyCode> = [
    58, // Left Option
    61, // Right Option
    59, // Left Control
    62, // Right Control
    63, // Fn
    54, // Right Command
    55, // Left Command
    56, // Left Shift
    60  // Right Shift
]

let formatter = DateFormatter()
formatter.dateFormat = "HH:mm:ss.SSS"

struct ModifierState {
    var lShift = false
    var lCtrl  = false
    var lOpt   = false
    var lCmd   = false

    var rShift = false
    var rCtrl  = false
    var rOpt   = false
    var rCmd   = false

    var fn   = false
    var caps = false
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
    57: "CAPS"
]

func keyName(_ keyCode: Int64) -> String {
    switch keyCode {
    // Letters
    case 0:  return "A"
    case 1:  return "S"
    case 2:  return "D"
    case 3:  return "F"
    case 4:  return "H"
    case 5:  return "G"
    case 6:  return "Z"
    case 7:  return "X"
    case 8:  return "C"
    case 9:  return "V"
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
    case 99:  return "F3"
    case 118: return "F4"
    case 96:  return "F5"
    case 97:  return "F6"
    case 98:  return "F7"
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

    if flags.contains(.maskShift) {
        result.append("SHIFT")
    }

    if flags.contains(.maskControl) {
        result.append("CTRL")
    }

    if flags.contains(.maskAlternate) {
        result.append("OPT")
    }

    if flags.contains(.maskCommand) {
        result.append("CMD")
    }

    if flags.contains(.maskSecondaryFn) {
        result.append("FN")
    }

    return result.isEmpty ? "NONE" : result.joined(separator: "+")
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

// MARK: - Modifier state tracking

//
// CGEvent flags are aggregate:
//
//   maskCommand = "some Command is down"
//
// They do NOT directly tell us:
//
//   "left Command is down"
//
// We therefore reconstruct left/right state from flagsChanged
// keycodes. This is useful diagnostic state, but it is NOT raw HID
// truth.
//

// Return the state of the *specific modifier key* represented by keyCode.
//
// IMPORTANT:
// Do NOT infer this from whether the aggregate Shift/Option/etc. flag
// changed. Another physical modifier of the same family may already
// be holding that aggregate flag.
//
// Instead, compare the event's aggregate flag against the previous
// per-key state we have recorded.
func updateModifierState(keyCode: Int64) -> String {
    let code = CGKeyCode(keyCode)

    switch code {
    case 56: // Left Shift
        state.lShift.toggle()
        return state.lShift ? "DOWN" : "UP"

    case 60: // Right Shift
        state.rShift.toggle()
        return state.rShift ? "DOWN" : "UP"

    case 59: // Left Control
        state.lCtrl.toggle()
        return state.lCtrl ? "DOWN" : "UP"

    case 62: // Right Control
        state.rCtrl.toggle()
        return state.rCtrl ? "DOWN" : "UP"

    case 58: // Left Option
        state.lOpt.toggle()
        return state.lOpt ? "DOWN" : "UP"

    case 61: // Right Option
        state.rOpt.toggle()
        return state.rOpt ? "DOWN" : "UP"

    case 55: // Left Command
        state.lCmd.toggle()
        return state.lCmd ? "DOWN" : "UP"

    case 54: // Right Command
        state.rCmd.toggle()
        return state.rCmd ? "DOWN" : "UP"

    case 63: // Fn
        state.fn.toggle()
        return state.fn ? "DOWN" : "UP"

    case 57: // Caps Lock
        state.caps.toggle()
        return state.caps ? "DOWN" : "UP"

    default:
        return "?"
    }
}

func keyState(_ down: Bool, _ symbol: String) -> String {
    down ? symbol : "·"
}

func printHeader() {
    print("")
    print(
        "TIME          " +
        "LS LC LO LM | " +
        "RS RC RO RM | " +
        "FN CP | " +
        "EVENT              " +
        "FLAGS"
    )

    print(
        "              " +
        "-- --------- | --------- | " +
        "-- -- | " +
        "------------------- " +
        "----------------"
    )

    fflush(stdout)
}

func printEvent(
    _ event: CGEvent,
    type: CGEventType,
    description: String,
    modifierDirection: String? = nil
) {
    let now = formatter.string(from: Date())

    let f = event.flags
    let session = CGEventSource.flagsState(.combinedSessionState)

    let keyCode =
        event.getIntegerValueField(.keyboardEventKeycode)

    let sourcePID =
        event.getIntegerValueField(.eventSourceUnixProcessID)

    let sourceUserData =
        event.getIntegerValueField(.eventSourceUserData)

    // Event type
    let typeName: String = {
        switch type {
        case .keyDown:
            return "KEYDOWN"
        case .keyUp:
            return "KEYUP"
        case .flagsChanged:
            return "FLAGS"
        default:
            return "TYPE[\(type.rawValue)]"
        }
    }()
    let arrow: String
    
    if let direction = modifierDirection {
        switch direction {
        case "DOWN":
            arrow = "↓"
        case "UP":
            arrow = "↑"
        default:
            arrow = "?"
        }
    } else {
        switch type {
        case .keyDown:
            arrow = "↓"
        case .keyUp:
            arrow = "↑"
        default:
            arrow = " "
        }
    }
    
    // Actual key/event name.
    let eventName: String
    
    switch type {
    case .flagsChanged:
        eventName = "\(arrow) \(description)"
    
    case .keyDown:
        eventName = "\(arrow) \(keyName(keyCode))"
    
    case .keyUp:
        eventName = "\(arrow) \(keyName(keyCode))"
    
    default:
        eventName = "\(arrow) \(description)"
    }

    // Left modifiers
    let left =
        "\(keyState(state.lShift, "⇧")) " +
        "\(keyState(state.lCtrl,  "⌃")) " +
        "\(keyState(state.lOpt,   "⌥")) " +
        "\(keyState(state.lCmd,   "⌘"))"

    // Right modifiers
    let right =
        "\(keyState(state.rShift, "⇧")) " +
        "\(keyState(state.rCtrl,  "⌃")) " +
        "\(keyState(state.rOpt,   "⌥")) " +
        "\(keyState(state.rCmd,   "⌘"))"

    // Fn / Caps
    let special =
        "\(keyState(state.fn,   "fn")) " +
        "\(keyState(state.caps, "⇪"))"

    // Actual aggregate flags from this CGEvent.
    let flagsText = heldModifiers(f)

    print(
        String(
            format:
                "%@  %-9@ | %-9@ | %-5@ | %-16@ | %-8@ " +
                "flags=%-12@ code=%-3lld raw=%-8lld " +
                "session=%-8lld pid=%-6lld data=%lld",
            now,
            left as NSString,
            right as NSString,
            special as NSString,
            eventName as NSString,
            typeName as NSString,
            flagsText as NSString,
            keyCode,
            f.rawValue,
            session.rawValue,
            sourcePID,
            sourceUserData
        )
    )

    fflush(stdout)
}

let mask =
    CGEventMask(1 << CGEventType.keyDown.rawValue) |
    CGEventMask(1 << CGEventType.keyUp.rawValue) |
    CGEventMask(1 << CGEventType.flagsChanged.rawValue)
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
                    "⚠️ NON-MODIFIER FLAGS EVENT: " +
                    "keycode=0 " +
                    "time=\(event.timestamp) " +
                    "flags=\(event.flags.rawValue)"
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
    
                print(
                    "⚠️ WATCHED " +
                    "\(direction) " +
                    "key=\(keyName(code)) " +
                    "code=\(code) " +
                    "time=\(event.timestamp) " +
                    "cmd[L]=\(leftCommandHeld) " +
                    "cmd[R]=\(rightCommandHeld) " +
                    "flags=\(event.flags.rawValue) " +
                    "session=\(sessionFlags) " +
                    "pid=\(sourcePID) " +
                    "state=\(sourceState) " +
                    "keyboard=\(keyboardType) " +
                    "repeat=\(autorepeat)"
                )
    
                fflush(stdout)
            }
    
            // --------------------------------------------------------
            // Normal event output
            // --------------------------------------------------------
    
            printEvent(
                event,
                type: type,
                description: name,
                modifierDirection: direction
            )
    
        case .keyDown:
    
            let name = keyName(code)
    
            printEvent(
                event,
                type: type,
                description: "KEYDOWN \(name)"
            )
    
        case .keyUp:
    
            let name = keyName(code)
    
            printEvent(
                event,
                type: type,
                description: "KEYUP   \(name)"
            )
    
        default:
            break
        }
    
        return Unmanaged.passUnretained(event)
    }

guard let tap = CGEvent.tapCreate(
    tap: .cgSessionEventTap,
    place: .headInsertEventTap,
    options: .listenOnly,
    eventsOfInterest: mask,
    callback: callback,
    userInfo: nil
) else {

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
    "Columns: LS LC LO LM = left modifiers, " +
    "RS RC RO RM = right modifiers, " +
    "FN CP = Fn/Caps"
)

print(
    "Event flags are aggregate CoreGraphics state; " +
    "left/right state is reconstructed from keycodes."
)

print("")

fflush(stdout)

import CoreGraphics
import Foundation

func isControlKeyPressed() -> Bool {
    let leftCtrlDown = CGEventSource.keyState(.combinedSessionState, key: 59)
    let rightCtrlDown = CGEventSource.keyState(.combinedSessionState, key: 62)
    return leftCtrlDown || rightCtrlDown
}

func getCurrentTimestamp() -> String {
    let formatter = DateFormatter()
    formatter.dateFormat = "HH:mm:ss.SSS"
    return formatter.string(from: Date())
}

print(
    "Columns: LS LC LO LM = left modifiers, " +
    "RS RC RO RM = right modifiers, " +
    "FN CP = Fn/Caps"
)

print(      
    "Event flags are aggregate CoreGraphics state; " +
    "left/right state is reconstructed from keycodes."
)

print("")
fflush(stdout)

let interval: TimeInterval = 3.0
let now = Date().timeIntervalSince1970
let nextAlignedTime = ceil(now / interval) * interval
let initialDelay = nextAlignedTime - now

let initialTimer = Timer.scheduledTimer(withTimeInterval: initialDelay, repeats: false) { _ in
    
    let checkState = {
        let status = isControlKeyPressed() ? "DOWN" : "UP"
        print("[\(getCurrentTimestamp())] Control key is \(status).")
        fflush(stdout)
    }
    
    // Execute the initial synchronized check immediately
    checkState()
    
    // Establish the permanent recurring timer
    let repeatingTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { _ in
        checkState()
    }
    
    // Attach the repeating timer to the CoreFoundation RunLoop
    RunLoop.current.add(repeatingTimer, forMode: .default)
}

CFRunLoopRun()
SWIFT
