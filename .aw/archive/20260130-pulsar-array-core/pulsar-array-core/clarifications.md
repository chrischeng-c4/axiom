---
change: pulsar-array-core
date: 2026-01-30
---

# Clarifications

## Q1: First Package
- **Question**: Which P0 package should we start with for cclab-pulsar?
- **Answer**: numpy - N-dimensional arrays as the foundation for all other packages
- **Rationale**: numpy is the foundation that pandas, scipy, and most other packages depend on. Starting here ensures a solid base.

## Q2: Strategy
- **Question**: What's your preferred implementation strategy?
- **Answer**: Pure Rust - no dependencies on existing Rust crates like ndarray/polars
- **Rationale**: Full control over the implementation, consistent with cclab philosophy of not depending on existing Rust packages.

## Q3: API Style
- **Question**: What's the target API style?
- **Answer**: Pythonic - match Python library APIs closely for easy migration
- **Rationale**: Users migrating from numpy should find familiar APIs, reducing learning curve.

## Q4: Structure
- **Question**: Should we set up the full cclab-pulsar crate structure now?
- **Answer**: Minimal - start with core/ only, add modules as needed
- **Rationale**: Avoid over-engineering. Build incrementally as features are implemented.

## Q5: Consolidation
- **Question**: Should overlapping packages be consolidated into unified modules?
- **Answer**: Yes, consolidate overlapping functionality into unified APIs
- **Rationale**: 41 packages have significant overlap. Unified modules reduce duplication and provide cleaner APIs.

## Q6: MVP Scope
- **Question**: What's the MVP for the first proposal?
- **Answer**: Array core only - N-dimensional arrays with basic operations
- **Rationale**: Foundation for everything else. Keep first proposal focused and achievable.

