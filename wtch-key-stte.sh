#!/bin/bash

set -e

DIR="${TMPDIR:-/tmp}/key-state-watcher"
mkdir -p "$DIR"

cat > "$DIR/main.swift" <<'SWIFT'
import CoreGraphics
import Foundation

let formatter = DateFormatter()
formatter.dateFormat = "HH:mm:ss.SSS"

let modifiers: [Int64: String] = [
    56: "LSHIFT", 60: "RSHIFT",
    59: "LCTRL",  62: "RCTRL",
    58: "LOPT",   61: "ROPT",
    55: "LCMD",   54: "RCMD",
    63: "FN",     57: "CAPS"
]

let keys: [Int64: String] = [
    0: "a", 1: "s", 2: "d", 3: "f",
    4: "h", 5: "g", 6: "z", 7: "x",
    8: "c", 9: "v", 11: "b", 12: "q",
    13: "w", 14: "e", 15: "r", 16: "y",
    17: "t", 31: "o", 32: "u", 34: "i",
    35: "p", 37: "l", 38: "j", 40: "k",
    45: "n", 46: "m",
    36: "RETURN", 48: "TAB", 49: "SPACE",
    51: "BACKSPACE", 53: "ESC"
]

func bit(_ flags: CGEventFlags, _ flag: CGEventFlags) -> String {
    flags.contains(flag) ? "t" : "f"
}

func log(_ event: CGEvent, _ description: String) {
    let f = event.flags
    let time = formatter.string(from: Date())

    var held: [String] = []

    if f.contains(.maskShift) {
        held.append("SHIFT")
    }
    if f.contains(.maskSecondaryFn) {
        held.append("FN")
    }
    if f.contains(.maskControl) {
        held.append("CTRL")
    }
    if f.contains(.maskCommand) {
        held.append("CMD")
    }
    if f.contains(.maskAlternate) {
        held.append("OPT")
    }

    let heldText = held.isEmpty
        ? "NONE"
        : held.joined(separator: "+")

    let state = String(
        format: "%@    %@    %@   %@    %@   %@  %-17@ %@",
        time,
        bit(f, .maskShift),
        bit(f, .maskSecondaryFn),
        bit(f, .maskControl),
        bit(f, .maskCommand),
        bit(f, .maskAlternate),
        description as NSString,
        heldText
    )

    print(state)
    fflush(stdout)
}

let mask =
    CGEventMask(1 << CGEventType.keyDown.rawValue) |
    CGEventMask(1 << CGEventType.keyUp.rawValue) |
    CGEventMask(1 << CGEventType.flagsChanged.rawValue)

let callback: CGEventTapCallBack = { _, type, event, _ in
    if type == .tapDisabledByTimeout ||
       type == .tapDisabledByUserInput {
        print("EVENT TAP DISABLED")
        fflush(stdout)
        return Unmanaged.passUnretained(event)
    }

    let code = event.getIntegerValueField(.keyboardEventKeycode)

    switch type {
    case .keyDown:
        let key = keys[code] ?? "code:\(code)"
        log(event, "KEYDOWN \(key)")

    case .keyUp:
        let key = keys[code] ?? "code:\(code)"
        log(event, "KEYUP   \(key)")

    case .flagsChanged:
        let key = modifiers[code] ?? "MOD:\(code)"
        log(event, "FLAGS   \(key)")

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
    fputs("Cannot create event tap. Check Input Monitoring permissions.\n", stderr)
    exit(1)
}

let source = CFMachPortCreateRunLoopSource(
    kCFAllocatorDefault, tap, 0
)

CFRunLoopAddSource(
    CFRunLoopGetCurrent(),
    source,
    .commonModes
)

CGEvent.tapEnable(tap: tap, enable: true)

print("TIME            SHIFT FN CTRL CMD OPT   EVENT")
fflush(stdout)

CFRunLoopRun()
SWIFT

swiftc "$DIR/main.swift" -o "$DIR/watcher"

# Hide characters typed into this terminal.
stty -echo
trap 'stty echo' EXIT

"$DIR/watcher"
