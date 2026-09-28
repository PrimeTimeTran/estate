import CoreGraphics
import Foundation

func isControlKeyPressed() -> Bool {
    let leftCtrlDown = CGEventSource.keyState(.combinedSessionState, key: 59)
    let rightCtrlDown = CGEventSource.keyState(.combinedSessionState, key: 62)
    return leftCtrlDown || rightCtrlDown
}

// Helper to format the current system time down to the millisecond
func getCurrentTimestamp() -> String {
    let formatter = DateFormatter()
    formatter.dateFormat = "HH:mm:ss.SSS"
    return formatter.string(from: Date())
}

print("Syncing with system clock... Press Ctrl+C to exit.")

// 1. Calculate how many seconds remain until the next clean 3-second mark
let interval: TimeInterval = 3.0
let now = Date().timeIntervalSince1970
let nextAlignedTime = ceil(now / interval) * interval
let initialDelay = nextAlignedTime - now

// 2. Start a one-time timer that fires precisely at the next aligned second 
let initialTimer = Timer.scheduledTimer(withTimeInterval: initialDelay, repeats: false) { _ in
    
    // 3. Define the actual recurring logic
    let checkState = {
        let status = isControlKeyPressed() ? "DOWN" : "UP"
        print("[\(getCurrentTimestamp())] Control key is \(status).")
    }
    
    // Run the very first check right on the synced mark
    checkState()
    
    // 4. Start the permanent repeating timer exactly synchronized with the wall clock
    let repeatingTimer = Timer.scheduledTimer(withTimeInterval: interval, repeats: true) { _ in
        checkState()
    }
    
    // Add the new repeating timer to the active run loop
    RunLoop.current.add(repeatingTimer, forMode: .default)
}

RunLoop.current.run()
