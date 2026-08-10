---
title: Estate Pipeline | 1
description: Bootstrap a Delta Integral Paradigm workspace.
version: 0.1
author: Loi Tran
---

# Global

---

kind: resource
path: ~/.estate
description: Global files shared by every workspace.

meta:
owner: system
visibility: public

notes:

- Installs the global DIP toolchain.
- Available to every workspace.

---

- core/
- daemon/
- dashboard/
- executable/
- ide/
- lsp/
- toolchain/
- README.md
- docs/
- errors/001.md

# Workspace

---

kind: resource
path: /Users/future/KB/project/crates/estate/tmp

description: Root workspace configuration.

notes:

- pkg.loi is the workspace manifest.
- index.json stores the semantic index.
- view.json stores reusable views.

---

- src/
- public/
- public/docs
- .estate
- src/hooks/
- src/main.py
- src/main.rs
- src/main.js
- src/main.ts
- public/docs/v0.0.1
- public/docs/v0.0.2
- public/docs/versions.json
- public/index.html
- .estate/settings.json
- .estate/index.json
- .estate/view.json
- pkg.loi

# Language Support

---

kind: resource
path: /Users/future/KB/project/crates/estate/tmp

---

- Cargo.toml
- go.mod
- go.sum
- package.json
- pyproject.toml

# Documentation

---

kind: resource
path: /Users/future/KB/project/crates/estate/tmp/public/docs

description: Core estate documentation.

notes:

- "(name) directories are logical groups."
- "They are hidden from generated routes."

---

- 0.preface.md
- 1.introduction.md
- 2.prologue.md
- 3.welcome.md
- 4.philosophy.md
- 4.philosophy.[zen].md
- 4.philosophy.[problem-space].md
- 4.philosophy.[pain-point].md
- 4.philosophy.[reality].md
- 4.philosophy.[friction].md
- 4.philosophy.[vision].md
- 5.analogy.[deck-of-cards].md
- 5.analogy.[analog-television].md
- 5.analogy.[carrier-pigeon].md
- 5.analogy.[open-closed-principle].md
- 6.goal.md
- 7.rationale.md
- 8.principles.md
- 9.architecture.md
- 10.specification.md
- 11.roadmap.md
- 12.guide.md
- 13.best-practices.[open-closed-principle].md
- 14.conventions.md
- 15.cheatsheet.md
- 16.examples.md
- 17.community.md
- 18.contributing.md
- 19.references.md
- 20.glossary.md
- v0.0.1/
- v0.0.2/
- versions.json

# Public Website

---

kind: resource
path: /Users/future/KB/project/crates/estate/tmp/public

description: Static showcase.

notes:

- Demonstrates exported documentation.
- Framework-independent output.

---

- assets/
- css/style.css
- js/main.js
- js/delta.js
- js/md.js
- js/mdx.js
- js/react.js
- js/vue.js
- static/
- index.html
