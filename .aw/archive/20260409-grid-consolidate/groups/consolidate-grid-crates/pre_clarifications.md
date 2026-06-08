---
change: grid-consolidate
group: consolidate-grid-crates
date: 2026-04-08
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should the server binary remain as a [[bin]] target inside cclab-grid, or should we create a thin cclab-grid-server binary crate that just depends on cclab-grid?
- **Answer**: Keep the server binary as a [[bin]] target inside cclab-grid. The main.rs is small and all server logic is in the server module. No need for a separate binary crate.

### Q2: General
- **Question**: Should heavy server dependencies (axum, tokio, tower, yrs, uuid, chrono, etc.) be behind a cargo feature flag?
- **Answer**: Yes, use feature flags. The 'server' feature gates server deps (axum, tower, tower-http, yrs, uuid, chrono, tracing-subscriber, dotenvy, anyhow, futures, cclab-kv). The 'db' feature gates db deps (cclab-wal, bincode, parking_lot). Tokio and tracing are shared between server and db so gate them behind both features. The core, formula, and history modules have no feature gates — they are always available.

### Q3: General
- **Question**: Should the cclab-grid-db Cargo.toml use workspace versions during consolidation?
- **Answer**: Yes, normalize to workspace versions where the workspace defines them (serde, serde_json, thiserror). Keep explicit versions for deps not in workspace.

### Q4: General
- **Question**: How should the public API be structured — re-export everything from lib.rs or require users to import from sub-modules?
- **Answer**: Each module (core, formula, history, db, server) is pub mod in lib.rs. Users import via cclab_grid::core::*, cclab_grid::formula::*, etc. No blanket re-exports from the crate root.

