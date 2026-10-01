import Foundation
import IOKit
import IOKit.hid

// MARK: - Helpers

// MARK: - HID Manager

// We deliberately don't call IOHIDDeviceOpen().
// The existing macOS HID event system owns the devices.
// We only want to observe input values.

// MARK: - Device matching

// MARK: - Input callback

// MARK: - Schedule manager

// MARK: - Open manager

func pageName(_ page: Int) -> String {
  switch page {
  case 0x01: return "GenericDesktop"
  case 0x07: return "Keyboard"
  case 0x09: return "Button"
  case 0x0C: return "Consumer"
  default:
    return String(format: "0x%04X", page)
  }
}
func keyboardUsageName(_ usage: Int) -> String {
  switch usage {
  case 0x04...0x1D:
    let scalar = UnicodeScalar(
      Int(Character("A").asciiValue!) + usage - 0x04
    )!
    return String(Character(scalar))

  case 0x1E: return "1"
  case 0x1F: return "2"
  case 0x20: return "3"
  case 0x21: return "4"
  case 0x22: return "5"
  case 0x23: return "6"
  case 0x24: return "7"
  case 0x25: return "8"
  case 0x26: return "9"
  case 0x27: return "0"

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
    return String(format: "KEY[0x%02X]", usage)
  }
}
func consumerUsageName(_ usage: Int) -> String {
  switch usage {
  case 0x006F: return "BRIGHTNESS_UP"
  case 0x0070: return "BRIGHTNESS_DOWN"

  default:
    return String(
      format: "CONSUMER[0x%04X]",
      usage
    )
  }
}
let manager = IOHIDManagerCreate(
  kCFAllocatorDefault,
  IOOptionBits(kIOHIDOptionsTypeNone)
)
IOHIDManagerSetDeviceMatching(
  manager,
  nil
)
IOHIDManagerRegisterDeviceMatchingCallback(
  manager,
  { _, _, _, device in

    let manufacturer =
      IOHIDDeviceGetProperty(
        device,
        kIOHIDManufacturerKey as CFString
      ) as? String

    let product =
      IOHIDDeviceGetProperty(
        device,
        kIOHIDProductKey as CFString
      ) as? String

    let vendorID =
      IOHIDDeviceGetProperty(
        device,
        kIOHIDVendorIDKey as CFString
      ) as? NSNumber

    let productID =
      IOHIDDeviceGetProperty(
        device,
        kIOHIDProductIDKey as CFString
      ) as? NSNumber

    print("")
    print("==========================================")
    print("DEVICE")
    print("==========================================")

    print("manufacturer: \(manufacturer ?? "?")")
    print("product:      \(product ?? "?")")

    print(
      String(
        format: "vendor:       0x%04X",
        vendorID?.intValue ?? 0
      )
    )

    print(
      String(
        format: "product id:   0x%04X",
        productID?.intValue ?? 0
      )
    )

    // IMPORTANT:
    // Do NOT call IOHIDDeviceOpen().
    //
    // The device is already opened by Apple's HID event
    // system. Opening it here would request exclusive access.

    let elements =
      IOHIDDeviceCopyMatchingElements(
        device,
        nil,
        IOOptionBits(kIOHIDOptionsTypeNone)
      ) as? [IOHIDElement] ?? []

    var interesting = 0

    for element in elements {

      let page =
        Int(IOHIDElementGetUsagePage(element))

      let usage =
        Int(IOHIDElementGetUsage(element))

      guard
        page == kHIDPage_KeyboardOrKeypad || page == kHIDPage_Consumer
      else {
        continue
      }

      interesting += 1

      let name: String

      if page == kHIDPage_KeyboardOrKeypad {
        name = keyboardUsageName(usage)
      } else {
        name = consumerUsageName(usage)
      }

      print(
        String(
          format:
            "  %-22@ page=%@ usage=0x%04X",
          name as NSString,
          pageName(page) as NSString,
          usage
        )
      )
    }

    print("interesting elements: \(interesting)")
    print("")

    // We don't open the device.
    // We only schedule it so the HID manager can observe it.

    IOHIDDeviceScheduleWithRunLoop(
      device,
      CFRunLoopGetMain(),
      CFRunLoopMode.defaultMode.rawValue
    )
  },
  nil
)
IOHIDManagerRegisterInputValueCallback(
  manager,
  { _, _, _, value in

    let element =
      IOHIDValueGetElement(value)

    let page =
      Int(IOHIDElementGetUsagePage(element))

    let usage =
      Int(IOHIDElementGetUsage(element))

    guard
      page == kHIDPage_KeyboardOrKeypad || page == kHIDPage_Consumer
    else {
      return
    }

    let intValue =
      IOHIDValueGetIntegerValue(value)

    let name: String

    if page == kHIDPage_KeyboardOrKeypad {
      name = keyboardUsageName(usage)
    } else {
      name = consumerUsageName(usage)
    }

    let timestamp =
      String(
        format: "%.6f",
        Date().timeIntervalSince1970
      )

    print(
      "\(timestamp) " + "\(pageName(page)) " + "\(name) " + String(format: "usage=0x%04X", usage)
        + " value=\(intValue)"
    )

    fflush(stdout)
  },
  nil
)
IOHIDManagerScheduleWithRunLoop(
  manager,
  CFRunLoopGetMain(),
  CFRunLoopMode.defaultMode.rawValue
)
let result =
  IOHIDManagerOpen(
    manager,
    IOOptionBits(kIOHIDOptionsTypeNone)
  )
guard result == kIOReturnSuccess else {
  print(
    String(
      format:
        "IOHIDManagerOpen failed: 0x%08X",
      result
    )
  )

  exit(1)
}
print("")
print("==========================================")
print(" RAW HID WATCHER")
print("==========================================")
print("")
print("Press Right Shift once.")
print("Press Enter once.")
print("Press Right Shift again.")
print("")
print("Then reproduce the brightness bug.")
print("")
print("Ctrl-C to exit.")
print("==========================================")
print("")
CFRunLoopRun()
