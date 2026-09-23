# Jitouch Process & Launch Control

These commands help inspect, launch, stop, and diagnose the Jitouch macOS application.

---

## Check if Jitouch is running

### Search all processes whose command contains `jitouch`

```sh
ps aux | grep -i jitouch
# pgrep -fl Jitouch cleaner
```

- `ps aux` → show running processes for all users
- `grep -i jitouch` → filter the output for `jitouch`
- `-i` → case-insensitive (`Jitouch`, `jitouch`, etc.)

**Useful for:** a quick human-readable check.

> Note: this can also show the `grep` command itself.

---

### Inspect one specific process

```sh
ps -o pid,ppid,stat,command -p 6967
```

Replace `6967` with the PID you want to inspect.

- `-p 6967` → inspect only PID `6967`
- `pid` → process ID
- `ppid` → parent process ID
- `stat` → process state
- `command` → command used to launch it

Example:

```text
PID   PPID  STAT  COMMAND
6967  1894  S+    .../Jitouch.app/Contents/MacOS/Jitouch
```

**Useful for:** understanding exactly which executable is running and who launched it.

---

### Find Jitouch processes by name

```sh
pgrep -fl Jitouch
```

This is usually the **best quick check**.

- `pgrep` → search running processes
- `-f` → search the entire command line, not just the process name
- `-l` → show the process name/command

Example:

```text
8723 .../Jitouch.app/Contents/MacOS/Jitouch
```

The number at the beginning is the **PID**.

---

### Get only the PID

```sh
pgrep -f Jitouch
```

Example:

```text
8723
```

This is useful when another command needs the PID.

For example:

```sh
kill "$(pgrep -f Jitouch)"
```

Be careful with this form if multiple Jitouch processes exist.

---

# Run the Jitouch executable directly

First define the executable:

```sh
APP="$HOME/Library/Developer/Xcode/DerivedData/Jitouch-awdrxkydbewiitbfccdshqkylzkq/Build/Products/Debug/Jitouch.app/Contents/MacOS/Jitouch"
```

`APP` is now a shell variable containing the path to the compiled Jitouch executable.

You can inspect it:

```sh
echo "$APP"
```

Then execute it:

```sh
"$APP"
```

This runs the **native compiled macOS executable**, not a `.js` file or `.sh` script.

Because Jitouch is a GUI application, this command normally stays attached to your terminal while Jitouch is running.

The shell prompt will return after Jitouch exits.

### Run it in the background

```sh
"$APP" &
```

The `&` tells the shell:

> Start this process, but don't wait for it to exit.

You can then continue using the same terminal.

---

# Stop Jitouch

## Normal shutdown

```sh
killall Jitouch
```

Ask macOS to terminate processes named `Jitouch`.

This is generally the first command to try.

---

## Stop by matching the executable path

```sh
pkill -f '/Jitouch.app/Contents/MacOS/Jitouch'
```

- `pkill` → terminate processes matching a pattern
- `-f` → match against the entire command line

This is useful when the process name isn't enough or when you want to target this particular Jitouch executable.

---

## Force-kill a specific PID

```sh
kill -9 6967
```

This sends `SIGKILL` to PID `6967`.

Use this when a normal shutdown doesn't work.

**Prefer:**

```sh
kill 6967
```

before:

```sh
kill -9 6967
```

`kill` gives the application a chance to shut down cleanly.

`kill -9` is the "stop this process immediately" option.

---

# Verify that it actually stopped

After killing Jitouch:

```sh
pgrep -fl Jitouch
```

If there is no output, there are no matching Jitouch processes.

This is a useful habit:

```text
kill
  ↓
pgrep
  ↓
verify it's actually gone
```

---

# launchctl

`launchctl` is different from the commands above.

The commands above operate directly on **processes**.

`launchctl` operates on **macOS launch services** — things that macOS starts, stops, and manages through `launchd`.

This matters if Jitouch is configured to automatically launch in the background.

---

## Stop/unload a user LaunchAgent

```sh
launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/_.jitouch_.plist
```

Breakdown:

```sh
$(id -u)
```

gets your current macOS user ID.

For example:

```text
501
```

So:

```sh
gui/$(id -u)
```

might become:

```text
gui/501
```

The full command therefore means approximately:

> Remove this LaunchAgent from my graphical user-session launchd domain.

The plist:

```text
~/Library/LaunchAgents/_.jitouch_.plist
```

is a per-user LaunchAgent configuration file.

---

## Check launchctl for Jitouch

```sh
launchctl list | grep -i jitouch
```

This asks launchctl for the jobs it knows about and filters the result for `jitouch`.

If something appears, Jitouch may be registered as a launchd job even if the actual process isn't currently running.

---

# Important distinction

There are really **three different things** you're inspecting:

### 1. The executable

```text
Jitouch.app/Contents/MacOS/Jitouch
```

This is the actual compiled native program.

---

### 2. The running process

```sh
pgrep -fl Jitouch
```

This tells you whether that executable currently has a running process.

For example:

```text
8723 .../Contents/MacOS/Jitouch
```

`8723` is the PID.

---

### 3. launchd configuration

```sh
launchctl list | grep -i jitouch
```

This tells you whether macOS's launch system has a Jitouch-related job registered.

A launchd job and a running process are **not the same thing**.

---

# My normal workflow

When developing Jitouch, I'd use this sequence:

```sh
# 1. Is it running?
pgrep -fl Jitouch

# 2. If necessary, stop it
killall Jitouch

# 3. Verify it stopped
pgrep -fl Jitouch

# 4. Launch the freshly built executable
"$APP"

# 5. From another terminal, inspect it
pgrep -fl Jitouch

# 6. When finished, stop it
killall Jitouch
```

Only reach for:

```sh
kill -9 <PID>
```

if normal termination isn't working.

And use:

```sh
launchctl ...
```

when you're specifically investigating **automatic/background launching by macOS**, rather than simply running the app yourself.

## Tail log file

```sh
tail -F /tmp/jitouch-test.json
```
