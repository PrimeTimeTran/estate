#ifndef HID_EVENT_SHIM_H
#define HID_EVENT_SHIM_H

#include <CoreFoundation/CoreFoundation.h>
#include <stdint.h>

typedef struct __IOHIDEventSystemClient *IOHIDEventSystemClientRef;
typedef struct __IOHIDEvent *IOHIDEventRef;
typedef struct __IOHIDEventQueue *IOHIDEventQueueRef;

typedef uint32_t IOHIDEventType;
typedef uint32_t IOHIDEventField;

typedef void (*HIDEventCallback)(
    uint32_t type,
    uint32_t usagePage,
    uint32_t usage,
    int32_t down,
    int32_t repeat
);

void hid_start(HIDEventCallback callback);

#endif