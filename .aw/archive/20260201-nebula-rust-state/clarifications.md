---
change: nebula-rust-state
date: 2026-02-01
---

# Clarifications

## Q1: Nested Tracking
- **Question**: How should nested dict/list changes be tracked?
- **Answer**: Field-level only - track only top-level field names, nested changes mark the parent field as dirty
- **Rationale**: Simpler implementation, lower memory usage, sufficient for most MongoDB use cases since $set typically updates at field level

## Q2: Rollback Support
- **Question**: Should original values be stored for rollback?
- **Answer**: Yes, full rollback support - store original values to support rollback() method
- **Rationale**: Enables reverting changes when needed, useful for transaction-like semantics

## Q3: Deep Copy
- **Question**: How to handle deep copy of Python objects?
- **Answer**: Serialize to BSON - Rust handles conversion, providing PyO3 objects to Python
- **Rationale**: Consistent with the pattern where Rust provides objects via PyO3 and Python is a thin wrapper. BSON conversion happens in Rust for efficient storage and comparison

