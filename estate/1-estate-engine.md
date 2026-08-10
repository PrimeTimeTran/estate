---
title: Estate Engine
description: Manage machines .estate directories recursively from global, personal to nested.
version: 1
---

The estate-engine does not own user data. It owns the interpretation, indexing, and communication layer over filesystem-defined estates.

# CLI MVP

---

kind: cli api
description: For the user to manage the tool

---

Commands:

- install: create global installation
- version: print version to cli
- start: start process
- stop: stop process
- restart: restart process
- build: build bun
- init: project
- current: print the path/description of the current instance.
- status: print daemon health
- logs: stream daemon logs
- doctor: verify installation/configuration

# Events

---

kind: ipc-protocol
description: For listning

---

Filesystem events:

- FileCreated
- FileModified
- FileDeleted
- DirectoryCreated
- EstateDiscovered
- EstateRemoved

Engine events:

- IndexUpdated
- CacheInvalidated
- CommandExecuted

# REST API

---

kind: ipc
description: For tools to talk to one another. Assumes daemon is on.

---

- create-bookmark: enables users to save important files/snippets/blocks for reuse.
- list-bookmarks: walk file system(FS) recursively until reaching root checking for personal "estate" or "kb" installation which gives "child estate" "more" knowledge/assets. Enables creation of a "panel" of "my snippets", etc.
- Should have "run" command to enable users to trigger FS events and get feedback. Bash script, python, rust, doesn't matter.
