#!/bin/bash

set -e

DIR="${TMPDIR:-/tmp}/key-state-watcher"
mkdir -p "$DIR"

cat > "$DIR/main.swift" <<'SWIFT'
import CoreGraphics
import Foundation

// MARK: - Time

let formatter = DateFormatter()
formatter.dateFormat = "HH:mm:ss.SSS"

// MARK: - Modifier state
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
    case 57: return "CAPS"
    default: return "UNKNOWN"
    }
}

// MARK: - Key names

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

let keys: [Int64: String] = [
    0: "a",
    1: "s",
    2: "d",
    3: "f",
    4: "h",
    5: "g",
    6: "z",
    7: "x",
    8: "c",
    9: "v",
    11: "b",
    12: "q",
    13: "w",
    14: "e",
    15: "r",
    16: "y",
    17: "t",
    18: "1",
    19: "2",
    20: "3",
    21: "4",
    22: "6",
    23: "5",
    24: "=",
    25: "9",
    26: "7",
    27: "-",
    28: "8",
    29: "0",
    30: "]",
    31: "o",
    32: "u",
    33: "[",
    34: "i",
    35: "p",
    36: "RETURN",
    37: "l",
    38: "j",
    39: "'",
    40: "k",
    41: ";",
    42: "\\",
    43: ",",
    44: "/",
    45: "n",
    46: "m",
    47: ".",
    48: "TAB",
    49: "SPACE",
    50: "`",
    51: "BACKSPACE",
    53: "ESC",
    54: "RCMD",
    55: "LCMD",
    56: "LSHIFT",
    57: "CAPS",
    58: "LOPT",
    59: "LCTRL",
    60: "RSHIFT",
    61: "ROPT",
    62: "RCTRL",
    63: "FN",

    // Arrows
    123: "LEFT",
    124: "RIGHT",
    125: "DOWN",
    126: "UP"
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
func updateModifierState(
    keyCode: Int64
) -> String {
    switch keyCode {
    case 56: // LEFT SHIFT
        let old = state.lShift
        state.lShift.toggle()
        return old ? "UP" : "DOWN"

    case 60: // RIGHT SHIFT
        let old = state.rShift
        state.rShift.toggle()
        return old ? "UP" : "DOWN"

    case 59: // LEFT CTRL
        let old = state.lCtrl
        state.lCtrl.toggle()
        return old ? "UP" : "DOWN"

    case 62: // RIGHT CTRL
        let old = state.rCtrl
        state.rCtrl.toggle()
        return old ? "UP" : "DOWN"

    case 58: // LEFT OPTION
        let old = state.lOpt
        state.lOpt.toggle()
        return old ? "UP" : "DOWN"

    case 61: // RIGHT OPTION
        let old = state.rOpt
        state.rOpt.toggle()
        return old ? "UP" : "DOWN"

    case 55: // LEFT COMMAND
        let old = state.lCmd
        state.lCmd.toggle()
        return old ? "UP" : "DOWN"

    case 54: // RIGHT COMMAND
        let old = state.rCmd
        state.rCmd.toggle()
        return old ? "UP" : "DOWN"

    case 63: // FN
        let old = state.fn
        state.fn.toggle()
        return old ? "UP" : "DOWN"

    case 57: // CAPS
        let old = state.caps
        state.caps.toggle()
        return old ? "UP" : "DOWN"

    default:
        return "?"
    }
}

func keyState(_ down: Bool, _ symbol: String) -> String {
    down ? symbol : "·"
}


// MARK: - Output

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

    // Direction
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

// MARK: - Event tap

let mask =
    CGEventMask(1 << CGEventType.keyDown.rawValue) |
    CGEventMask(1 << CGEventType.keyUp.rawValue) |
    CGEventMask(1 << CGEventType.flagsChanged.rawValue)

let callback: CGEventTapCallBack = {
    _, type, event, _ in

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

    let code =
        event.getIntegerValueField(.keyboardEventKeycode)

    switch type {

    case .flagsChanged:
        let name = modifierName(code)
        let direction = updateModifierState(
            keyCode: code
        )
    
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

// MARK: - Create tap

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

// MARK: - Run loop

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

CFRunLoopRun()
SWIFT

swiftc "$DIR/main.swift" -o "$DIR/watcher"

# Hide characters typed into this terminal.
stty -echo
trap 'stty echo' EXIT

"$DIR/watcher"
