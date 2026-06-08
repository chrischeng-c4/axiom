---
change: jet-remaining
group: production-build-pipeline
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: Minifier Choice
- **Answer**: oxc_minifier is the preferred implementation for Milestone 3, as it is Rust-native and provides modern, high-performance minification that aligns with the rest of the cclab-jet codebase.

### Q2: CSS Strategy
- **Answer**: The CSS pipeline should handle @import resolution and PostCSS integration natively (leveraging Rust-based tools like lightningcss if possible) to minimize external CLI dependencies and maximize performance. Tailwind support should be handled via its JIT engine integration.

### Q3: Asset Hashing
- **Answer**: Yes, the [name].[hash].[ext] pattern should be the default for all assets (scripts, styles, and static assets) to ensure cache busting across deployments. Configuration should allow overriding this for specific asset types if needed.

