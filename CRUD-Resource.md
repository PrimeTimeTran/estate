## 1. Resource lifecycle

The fundamental CRUD:

- `resource create`

  - Create a new `ResourceId`
  - Register `ResourceKind`
  - Optionally attach initial location
  - Return the canonical `ResourceId`

- `resource get`

  - Fetch resource by `ResourceId`
  - Return basic metadata

- `resource update`

  - Change mutable metadata
  - Add/remove locations
  - Update resource properties

- `resource delete`

  - Remove resource
  - Decide whether aliases/anchors/edges cascade
  - Preserve history if desired

- `resource list`

  - Query resources
  - Filter by kind/location/etc.
  - Sort/paginate

- `resource inspect`

  - Return the complete known representation of a resource

This is the **Registry/Store foundation**.

---

# 2. Alias lifecycle

Aliases are your:

```text
@my-pipeline
@build
@foo
```

CRUD:

- `alias create`

  - `alias create foo 55`
  - Validate uniqueness
  - Bind alias → `ResourceId`

- `alias get`

  - Resolve exact alias

- `alias update`

  - Rename alias
  - Rebind alias

- `alias delete`

  - Remove binding

- `alias list`

  - List aliases for resource
  - List all aliases

- `alias resolve`

  - `@foo → ResourceId(55)`

This becomes one of the first implementations of the **Resolver**.

---

# 3. Anchor lifecycle

Anchors are slightly different from aliases.

Think:

```text
@pipeline#build
@document#installation
```

Stages:

- `anchor create`

  - Resource + anchor name
  - Eventually target/position

- `anchor get`

  - Retrieve anchor

- `anchor update`

  - Rename
  - Change target

- `anchor delete`
- `anchor list`

  - Anchors belonging to resource

- `anchor resolve`

  - `@foo#build → ResourceId + AnchorTarget`

Initially your `Anchor` can be simple:

```rust
Anchor {
    name,
    resource,
}
```

Later:

```text
Anchor
  ├── resource
  ├── name
  └── target
       ├── line
       ├── byte offset
       ├── symbol
       └── selector
```

---

# 4. Location lifecycle

This is the VFS boundary.

CRUD-ish operations:

- `location add`

  - Resource → File/Git/Remote

- `location get`
- `location update`
- `location remove`
- `location list`
- `location resolve`

  - Resource → best usable location

Important distinction:

```text
Resolver:
    @foo → ResourceId

Location resolver:
    ResourceId → ResourceLocation

VFS:
    ResourceLocation → access
```

That separation will save you later.

---

# 5. Graph / relationship lifecycle

Your:

```rust
Edge {
    from,
    to,
    kind,
}
```

needs:

- `edge create`

  - Resource A → Resource B
  - `depends_on`
  - `contains`
  - `references`
  - etc.

- `edge get`
- `edge update`
- `edge delete`
- `edge list`

  - outgoing
  - incoming

- `graph`

  - Show neighborhood

- `graph ancestors`
- `graph descendants`
- `graph path`

Eventually:

```text
estate graph @foo
```

could produce:

```text
foo
├── depends_on → bar
├── references → baz
└── contains
    ├── qux
    └── quux
```

---

# 6. Resolution

This is arguably the **most important subsystem**.

You eventually want:

```text
Input
  │
  ├── ResourceId
  ├── @alias
  ├── @alias#anchor
  ├── filesystem path
  ├── URI
  ├── Git reference
  ├── wikilink
  └── perhaps URI schemes
        │
        ▼
     Resolver
        │
        ▼
 ResourceReference
        │
        ▼
    ResourceId
```

Commands:

- `resolve <reference>`
- `resolve --json`
- `resolve --explain`
- `resolve --all`

`--explain` would be particularly useful for debugging:

```text
Input:
    @build

Parsed as:
    Alias

Alias:
    build

Resolved:
    Resource #55

Location:
    /project/.estate/pipelines/build.json
```

That will become **extremely valuable** once LSP/IDE integrations exist.

---

# 7. VFS operations

Once resolution works:

- `open`
- `read`
- `write`
- `stat`
- `exists`
- `copy`
- `move`
- `delete`
- `watch`

But I wouldn't necessarily expose all of these as CLI commands.

The important thing is that the **Rust API exists**.

For example:

```rust
vfs.open(resource_id)
vfs.read(resource_id)
vfs.stat(resource_id)
vfs.watch(resource_id)
```

CLI can expose only the useful subset.

LSP/IDE can use the richer API.

---

# 8. Discovery / indexing

This is how the Registry gets populated.

Commands:

```text
estate scan
estate index
estate rebuild
estate refresh
```

Stages:

- Walk filesystem
- Apply exclusions
- Identify resources
- Extract metadata
- Discover locations
- Discover aliases
- Discover anchors
- Discover relationships
- Compare against existing registry
- Insert/update/remove records
- Record scan metadata

You eventually want **incremental indexing**, not rebuilding everything.

Something like:

```text
Filesystem
    ↓
Watcher
    ↓
Changed path
    ↓
Discovery
    ↓
Resource update
    ↓
Registry
```

---

# 9. Watcher / synchronization

Eventually:

```bash
estate watch
```

or daemon-internal only.

Stages:

- Watch filesystem
- Detect create
- Detect modify
- Detect delete
- Detect rename
- Re-scan affected resource
- Update registry
- Update graph
- Notify subscribers

This becomes important for:

- VS Code
- Zed
- LSP
- web UI
- background daemon

---

# 10. Query system

Don't make every consumer invent SQL queries.

Build a domain query layer.

Examples:

```text
find resources by kind
find resources by alias
find resources by location
find resources by path
find resources referencing X
find resources contained by X
find resources changed since X
```

Eventually:

```rust
store.query(ResourceQuery {
    kind: Some(ResourceKind::Document),
    ...
})
```

Then:

```text
CLI ───────┐
LSP ───────┤
VS Code ───┤
Zed ───────┼──→ Query API
Web ───────┤
Scripts ───┘
```

This is much better than making each integration talk directly to SQLite.

---

# 11. Import / export

This is where your earlier JSON idea becomes useful.

Commands:

```bash
estate export
estate import
```

Formats:

- JSON
- JSONL
- potentially TOML
- eventually SQLite database backup

Useful for:

- debugging
- migrations
- backups
- scripts
- web applications
- moving an Estate
- inspecting the registry manually

And:

```bash
estate resource list --json
```

should probably be standard.

---

# 12. Transactions / mutations

Once multiple things can mutate the registry, you need atomic operations.

For example:

```text
create resource
    ↓
add location
    ↓
create alias
    ↓
create edges
```

should ideally be one transaction.

Something like:

```rust
estate.transaction(|tx| {
    let id = tx.create_resource(...)?;
    tx.add_location(id, ...)?;
    tx.create_alias(...)?;
    Ok(())
})?;
```

This becomes especially important when the daemon is serving multiple clients.

---

# 13. Events

This is where the architecture starts becoming really powerful.

When:

```text
Resource #55 changed
```

the engine can emit:

```text
ResourceCreated
ResourceUpdated
ResourceDeleted

AliasCreated
AliasDeleted

LocationAdded
LocationRemoved

EdgeCreated
EdgeDeleted
```

Consumers can subscribe:

```text
                    Estate Engine
                         │
                    Event Stream
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
        LSP           VS Code          Zed
```

This is how you avoid every integration constantly polling the database.

---

# 14. Daemon/API layer

Your daemon then becomes essentially:

```text
                    Estate Core
                         │
              ┌──────────┴──────────┐
              │                     │
           Local IPC              Library
              │                     │
          CLI / LSP             Scripts / Rust
```

The daemon should not contain your business logic.

Ideally:

```text
CLI
 ↓
IPC
 ↓
Daemon
 ↓
EstateService
 ↓
Resolver / Store / VFS / Graph
```

And:

```text
Rust binary
 ↓
EstateService
```

can bypass the daemon if it wants.

That's important for your "general purpose scripts/binaries" goal.

---

# 15. Capability / permission model

Probably later, but worth leaving room for:

- read resource
- write resource
- modify aliases
- modify graph
- modify filesystem
- execute actions

Especially if the daemon eventually becomes a long-running service.

---

# 16. CLI surface I'd eventually aim toward

Not necessarily implement all at once:

```text
estate
├── resource
│   ├── create
│   ├── get
│   ├── update
│   ├── delete
│   ├── list
│   └── inspect
│
├── alias
│   ├── create
│   ├── get
│   ├── update
│   ├── delete
│   ├── list
│   └── resolve
│
├── anchor
│   ├── create
│   ├── get
│   ├── update
│   ├── delete
│   ├── list
│   └── resolve
│
├── location
│   ├── add
│   ├── get
│   ├── update
│   ├── remove
│   └── list
│
├── edge
│   ├── create
│   ├── get
│   ├── update
│   ├── delete
│   └── list
│
├── resolve
│
├── graph
│   ├── show
│   ├── ancestors
│   ├── descendants
│   └── path
│
├── scan
├── watch
├── query
├── export
├── import
├── doctor
└── status
```

But **don't implement that entire tree now**.

---

# The actual roadmap I'd use

I'd make the stages:

### Phase 1 — Identity

- `Resource`
- `ResourceId`
- `ResourceKind`
- `ResourceStore`
- SQLite persistence
- `resource create/get/list`

### Phase 2 — References

- `Alias`
- `Anchor`
- Resolver
- `alias create/get/list/resolve`
- `anchor create/get/list/resolve`

### Phase 3 — Locations

- `ResourceLocation`
- Location store
- `location add/list`
- Resource → location resolution

### Phase 4 — VFS

- `open`
- `read`
- `stat`
- `watch`
- ResourceId → accessible resource

### Phase 5 — Graph

- `Edge`
- edge CRUD
- graph traversal/query

### Phase 6 — Discovery

- filesystem scanner
- registry indexing
- incremental updates
- reconciliation

### Phase 7 — Events

- resource events
- watcher events
- subscriptions
- daemon notifications

### Phase 8 — External consumers

- CLI
- JSON API
- LSP
- VS Code adapter
- Zed adapter
- Rust library/API
- scripts

### Phase 9 — Advanced

- transactions
- migrations
- import/export
- caching
- permissions
- remote resources
- distributed/sync behavior if you ever need it

---

## And I'd make your first milestone **very small**

I wouldn't start with "implement CRUD."

I'd literally make this work:

```bash
estate anchor create foo
estate anchor list
estate resolve @foo
estate anchor delete foo
```

with:

```text
CLI
 ↓
Daemon
 ↓
AnchorService
 ↓
Resolver
 ↓
ResourceStore
 ↓
SQLite
```

Then:

```bash
estate resolve @foo --json
```

becomes the first **machine-consumable API**.

Once that works, you've got the skeleton that everything else can hang off of.

And importantly, **don't make the CLI the API**. Make the Rust service/domain API the API, make the daemon an IPC transport for it, and make CLI/LSP/IDE integrations clients of that same API. That's what will keep your "do it once, consume it everywhere" goal intact.
