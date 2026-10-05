Rust Debugging
├── Knowledge
│   ├── compiler errors
│   ├── ownership / borrowing
│   ├── async Send/Sync
│   └── trait resolution
│
├── Heuristics
│   ├── read the first meaningful compiler error
│   ├── reduce the failing expression
│   ├── inspect inferred types
│   └── verify assumptions with cargo check
│
├── Tools
│   ├── cargo check
│   ├── cargo test
│   ├── cargo tree
│   ├── rustc
│   └── rust-analyzer
│
├── Constraints
│   ├── don't blindly clone to satisfy borrow checker
│   └── don't suppress errors without understanding them
│
└── Patterns
    ├── isolate trait bounds
    ├── inspect lifetime relationships
    └── bisect async Send failures