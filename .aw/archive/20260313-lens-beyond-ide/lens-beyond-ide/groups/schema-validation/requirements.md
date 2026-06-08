---
change: lens-beyond-ide
group: schema-validation
date: 2026-03-13
---

# Requirements

Bundle JSON schemas for K8s (1.28, 1.29, 1.30) and GitLab CI via include_bytes!() for offline validation.

1. CREATE src/schemas/mod.rs — SchemaRegistry with validate_k8s(value, version) and validate_gitlab_ci(value)
2. Bundle schema files (compressed or raw JSON) as static bytes
3. Wire K8s checker rules K8002, K8008 to use schema validation
4. Wire GitLab CI checker rule GL002 to use schema validation
5. Add --k8s-version CLI flag for version selection (default: 1.30)
6. Add jsonschema crate dependency if not already present
