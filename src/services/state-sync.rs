// # Goal
//
// Create a small daemon that watches a project's Cargo.toml and keeps
// Rust IDE configuration synchronized with the project's Cargo configuration.
//
// Cargo.toml is the source of truth.
//
// ## Scope
//
// Watch:
// - Cargo.toml
//
// Synchronize:
// - VSCode settings.json
// - Zed settings.json
//
// ## IDE Configuration
//
// ### VSCode
//
// File:
// ~/Library/Application Support/Code/User/settings.json
//
// Relevant settings:
//
// {
//   "rust-analyzer.cargo.features": ["native"],
//   "rust-analyzer.cargo.target": "aarch64-apple-darwin"
// }
//
// ### Zed
//
// File:
// ~/.config/zed/settings.json
//
// Relevant settings:
//
// {
//   "lsp": {
//     "rust-analyzer": {
//       "initialization_options": {
//         "linkedProjects": [
//           "/Users/future/kb/project/Cargo.toml"
//         ],
//         "cargo": {
//           "features": ["native"]
//         }
//       }
//     }
//   }
// }
//
// ## Synchronization Rules
//
// Cargo.toml is authoritative.
//
// When Cargo.toml changes:
//
// 1. Parse Cargo.toml.
// 2. Determine the active Rust configuration.
// 3. Update the corresponding IDE settings.
// 4. Preserve all unrelated IDE settings.
//
// The daemon must modify only the settings it owns.
//
// ## Configuration Mapping
//
// Cargo features:
// - Cargo `[features]` -> IDE rust-analyzer cargo features.
//
// Project:
// - Cargo.toml path -> Zed `linkedProjects`.
//
// Target:
// - Explicitly configured by the daemon/project configuration.
// - Do not infer the target from Cargo.toml unless a deterministic rule exists.
//
// ## Runtime Behavior
//
// - Start the daemon once.
// - Watch Cargo.toml for filesystem changes.
// - Debounce rapid filesystem events.
// - Synchronize immediately on startup.
// - Re-synchronize whenever Cargo.toml changes.
// - Do not rewrite settings files when their effective contents have not changed.
//
// ## Safety
//
// - Parse JSON before modifying settings.
// - Preserve unrelated settings.
// - Write settings atomically.
// - Never leave a partially-written settings file.
// - Handle missing settings files gracefully.
//
// ## Future
//
// Support:
// - multiple projects
// - multiple Cargo.toml files
// - project-specific configuration
// - VSCode/Zed enable/disable
// - additional IDEs

fn main() {}
