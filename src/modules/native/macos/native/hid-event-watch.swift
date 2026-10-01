import Foundation

@_silgen_name("hid_start")
func hid_start(
  _ callback:
    @escaping @convention(c) (
      UInt32,
      UInt32,
      UInt32,
      Int32,
      Int32
    ) -> Void
)
func keyboardName(
  usagePage: UInt32,
  usage: UInt32
) -> String {

  guard usagePage == 0x07 else {
    return String(
      format: "PAGE[0x%04X]/USAGE[0x%04X]",
      usagePage,
      usage
    )
  }

  switch usage {
  case 0x04...0x1D:
    let scalar =
      UnicodeScalar(
        Int(Character("A").asciiValue!) + Int(usage) - 0x04
      )!

    return String(Character(scalar))

  case 0x28: return "ENTER"
  case 0x29: return "ESC"
  case 0x2A: return "BACKSPACE"
  case 0x2B: return "TAB"
  case 0x2C: return "SPACE"

  case 0xE0: return "LCTRL"
  case 0xE1: return "LSHIFT"
  case 0xE2: return "LOPT"
  case 0xE3: return "LCMD"

  case 0xE4: return "RCTRL"
  case 0xE5: return "RSHIFT"
  case 0xE6: return "ROPT"
  case 0xE7: return "RCMD"

  default:
    return String(
      format: "KEY[0x%02X]",
      usage
    )
  }
}
let callback:
  @convention(c) (
    UInt32,
    UInt32,
    UInt32,
    Int32,
    Int32
  ) -> Void = {
    _, usagePage, usage, down, isRepeat in

    print(
      "page=\(usagePage) " + "usage=\(usage) " + "down=\(down) " + "repeat=\(isRepeat)"
    )
  }
print("")
print("==========================================")
print(" HID EVENT SYSTEM WATCHER")
print("==========================================")
print("")
print("Watching Apple's HID event system.")
print("")
print("Press:")
print("  Right Shift")
print("  Enter")
print("  Right Shift")
print("")
print("Then reproduce the brightness problem.")
print("")
print("Ctrl-C to exit.")
print("==========================================")
print("")
hid_start(callback)
