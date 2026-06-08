---
number: 189
title: "Shield performance: track remaining gaps vs pydantic v2"
state: open
labels: [enhancement, crate:schema, P1]
---

# #189 — Shield performance: track remaining gaps vs pydantic v2

## Current State

Shield validation module has been optimized with Rust-backed type checking and fast construction paths. Summary of performance vs pydantic v2 (+orjson):

| Scenario | Small (5 fields) | Medium (10+2 nested) | Large (20+nested) |
|---|---|---|---|
| **dict->model** | 1.14x | 1.30x | 1.23x |
| **model->dict** | **0.55x** ✅ | **0.67x** ✅ | 1.11x |
| **json->model** | 1.48x | 1.82x | 1.70x |
| **model->json** | 1.08x | 1.02x | 1.72x |

## What's Been Done

- [x] Rust `validate_types_fast` — lightweight type checking via `is_instance_of` (no Value conversion)
- [x] Pre-computed class metadata (`_field_names`, `_type_tags`, `_static_defaults`, `_nested_fields`)
- [x] Skip full schema validation when no Field constraints (only type check)
- [x] Static defaults fast path for simple models (dict.copy + update)
- [x] Mutable default isolation (auto-convert list/dict/set to copy-factories)
- [x] Rust `construct_model_dict` (implemented but not wired — FFI overhead made it slower)
- [x] Inlined `__init__` fast path to reduce Python dispatch overhead

## Production Readiness Assessment

**Ready for general use:** dict->model and model->dict are at or near pydantic parity. model->json is competitive for small/medium models.

**Caution for high-throughput JSON APIs:** json->model (1.5-1.8x) is the main gap. This is architectural — pydantic-core constructs nested models entirely in Rust, while shield crosses the Python↔Rust FFI boundary for each nested model's `__new__` + `__dict__` assignment.

## Remaining Architectural Limitations

The core bottleneck is **nested model construction crossing the FFI boundary**:
- Each nested model requires: Python `cls.__new__(cls)` → Rust field loop → Python `__dict__` assignment
- Pydantic-core avoids this by storing model data in Rust-native structures
- Closing this gap would require storing model data in Rust (major architecture change)

## Potential Future Optimizations

1. **Rust-native model storage** — Store field data in Rust `PyClassNativeType` instead of Python `__dict__`. Would eliminate FFI crossings for nested models but requires significant refactoring.
2. **Combined validate+construct** — Single Rust function that parses JSON → validates → constructs model graph. Would help json->model specifically.
3. **Per-class cached type descriptors** — Cache parsed `TypeDescriptor` in Rust (static HashMap keyed by class id) to avoid re-parsing schema on every `validate` call.

## Benchmark Reproduction

```bash
maturin develop --release
uv run python3 python/tests/shield/benchmarks/bench_comprehensive.py
```
