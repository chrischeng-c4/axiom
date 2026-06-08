---
change: pulsar-frame
date: 2026-01-30
---

# Clarifications

## Q1: Feature Scope
- **Question**: What DataFrame features should pulsar-frame include for MVP?
- **Answer**: Full pandas API with potential cclab.shield extension for validation features
- **Rationale**: Comprehensive pandas compatibility ensures easy migration from Python. Shield integration can provide schema validation for data quality.

## Q2: Internal Storage
- **Question**: Should pulsar-frame depend on pulsar-array internally?
- **Answer**: Yes, use pulsar-array as internal storage backend
- **Rationale**: Leverages existing N-dimensional array implementation, ensures consistency across Pulsar ecosystem, and avoids code duplication.

## Q3: Index Types
- **Question**: What index types should be supported?
- **Answer**: String + Integer indexes
- **Rationale**: Matches pandas behavior where DataFrames have both positional (iloc) and label-based (loc) indexing. Essential for pandas API compatibility.

