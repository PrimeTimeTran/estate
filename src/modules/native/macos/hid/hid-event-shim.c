#include "hid-event-shim.h"
#include <dlfcn.h>
#include <stdio.h>

static HIDEventCallback gCallback = NULL;

typedef IOHIDEventSystemClientRef (*ClientCreateFn)(
    CFAllocatorRef allocator
);

typedef void (*ScheduleFn)(
    IOHIDEventSystemClientRef client,
    CFRunLoopRef runLoop,
    CFStringRef mode
);

typedef void (*RegisterCallbackFn)(
    IOHIDEventSystemClientRef client,
    void (*callback)(
        void *,
        void *,
        IOHIDEventQueueRef,
        IOHIDEventRef
    ),
    void *target,
    void *refcon
);

typedef uint32_t (*GetTypeFn)(
    IOHIDEventRef event
);

typedef int64_t (*GetIntegerValueFn)(
    IOHIDEventRef event,
    IOHIDEventField field,
    int options
);

static GetTypeFn gGetType;
static GetIntegerValueFn gGetIntegerValue;

static void event_callback(
    void *target,
    void *refcon,
    IOHIDEventQueueRef queue,
    IOHIDEventRef event
) {
    if (!event || !gCallback) {
        return;
    }

    uint32_t type =
        gGetType(event);

    /*
     * kIOHIDEventTypeKeyboard = 3
     *
     * We deliberately avoid depending on the private
     * enum/header here.
     */
    if (type != 3) {
        return;
    }

    /*
     * These are the keyboard event fields used by
     * IOHIDEvent.
     */
    const IOHIDEventField usagePageField = 3;
    const IOHIDEventField usageField     = 4;
    const IOHIDEventField downField      = 5;
    const IOHIDEventField repeatField    = 6;

    uint32_t usagePage =
        (uint32_t)gGetIntegerValue(
            event,
            usagePageField,
            0
        );

    uint32_t usage =
        (uint32_t)gGetIntegerValue(
            event,
            usageField,
            0
        );

    int32_t down =
        (int32_t)gGetIntegerValue(
            event,
            downField,
            0
        );

    int32_t isRepeat =
        (int32_t)gGetIntegerValue(
            event,
            repeatField,
            0
        );

    gCallback(
        type,
        usagePage,
        usage,
        down,
        isRepeat
    );
}

void hid_start(HIDEventCallback callback) {

    gCallback = callback;

    void *handle =
        dlopen(
            "/System/Library/Frameworks/IOKit.framework/IOKit",
            RTLD_LAZY
        );

    if (!handle) {
        fprintf(
            stderr,
            "dlopen IOKit failed: %s\n",
            dlerror()
        );
        return;
    }

    ClientCreateFn clientCreate =
        (ClientCreateFn)dlsym(
            handle,
            "IOHIDEventSystemClientCreate"
        );

    ScheduleFn schedule =
        (ScheduleFn)dlsym(
            handle,
            "IOHIDEventSystemClientScheduleWithRunLoop"
        );

    RegisterCallbackFn registerCallback =
        (RegisterCallbackFn)dlsym(
            handle,
            "IOHIDEventSystemClientRegisterEventCallback"
        );

    gGetType =
        (GetTypeFn)dlsym(
            handle,
            "IOHIDEventGetType"
        );

    gGetIntegerValue =
        (GetIntegerValueFn)dlsym(
            handle,
            "IOHIDEventGetIntegerValue"
        );

    if (!clientCreate ||
        !schedule ||
        !registerCallback ||
        !gGetType ||
        !gGetIntegerValue) {

        fprintf(
            stderr,
            "Could not resolve IOHID symbols.\n"
        );

        return;
    }

    IOHIDEventSystemClientRef client =
        clientCreate(
            kCFAllocatorDefault
        );

    if (!client) {
        fprintf(
            stderr,
            "IOHIDEventSystemClientCreate failed.\n"
        );
        return;
    }

    schedule(
        client,
        CFRunLoopGetCurrent(),
        kCFRunLoopDefaultMode
    );

    registerCallback(
        client,
        event_callback,
        NULL,
        NULL
    );

    printf(
        "IOHIDEventSystemClient active.\n"
    );

    fflush(stdout);

    CFRunLoopRun();
}


