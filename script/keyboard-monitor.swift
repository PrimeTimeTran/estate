import Cocoa

let keyNames: [UInt16: String] = [
    55: "Left Command", 54: "Right Command",
    56: "Left Shift", 60: "Right Shift",
    58: "Left Option", 61: "Right Option",
    59: "Left Control", 62: "Right Control",
    57: "Caps Lock", 63: "Fn"
]

func modifiers(_ flags: NSEvent.ModifierFlags) -> String {
    var names: [String] = []
    if flags.contains(.command) { names.append("CMD") }
    if flags.contains(.shift) { names.append("SHIFT") }
    if flags.contains(.option) { names.append("OPT") }
    if flags.contains(.control) { names.append("CTRL") }
    if flags.contains(.function) { names.append("FN") }
    return names.joined(separator: "+")
}

let formatter = DateFormatter()
formatter.dateFormat = "HH:mm:ss.SSS"

func record(_ event: NSEvent) {
    let kind: String

    switch event.type {
    case .keyDown: kind = "DOWN"
    case .keyUp: kind = "UP"
    case .flagsChanged: kind = "MODIFIER"
    default: return
    }

    let name = keyNames[event.keyCode] ?? "Key \(event.keyCode)"
    let time = formatter.string(from: Date())

    print("\(time) \(kind) \(name) [\(modifiers(event.modifierFlags))]")
    fflush(stdout)
}

guard let tap = CGEvent.tapCreate(
    tap: .cgSessionEventTap,
    place: .headInsertEventTap,
    options: .listenOnly,
    eventsOfInterest:
        (1 << CGEventType.keyDown.rawValue) |
        (1 << CGEventType.keyUp.rawValue) |
        (1 << CGEventType.flagsChanged.rawValue),
    callback: { _, _, event, _ in
        if let nsEvent = NSEvent(cgEvent: event) {
            record(nsEvent)
        }
        return Unmanaged.passUnretained(event)
    },
    userInfo: nil
) else {
    print("Cannot monitor keyboard. Check Terminal's Input Monitoring permission.")
    exit(1)
}

let source = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)
CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
CGEvent.tapEnable(tap: tap, enable: true)

print("Monitoring keyboard. Press Ctrl+C to stop.")
CFRunLoopRun()
