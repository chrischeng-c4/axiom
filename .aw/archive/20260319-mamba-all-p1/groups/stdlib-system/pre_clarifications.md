---
change: mamba-all-p1
group: stdlib-system
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: HYBRID: Atomic RC (rc.rs with AtomicU32) + cycle-detecting tracing GC (gc.rs, 14KB). Mark-sweep with safepoint protocol, threshold-based collection at 700 allocs. gc.collect() should trigger a full mark-sweep cycle. gc.disable()/enable() can toggle the automatic threshold-based trigger. No gc stdlib module yet but the infrastructure is complete.

### Q2: General
- **Answer**: YES. Module registry exists at module.rs with thread-local MODULES HashMap. MbModule has name, file, attrs, is_package. mb_import() with circular import protection (sentinel pre-caching). mb_import_from() for selective imports. Search path support with find_module(). importlib.import_module() can delegate to existing mb_import(). reload() would need to clear and re-execute module body.

### Q3: General
- **Answer**: No allocator hooks exposed. gc.rs tracks container objects (list, dict, instance) in a global set but doesn't track per-object allocation sizes. Recommend coarse memory stats initially (total tracked objects, number of collections) rather than exact per-object tracking. Full tracemalloc fidelity would require adding size tracking to MbObjectHeader.

### Q4: General
- **Answer**: type() builtin works (builtins.rs:475) returning Instance with __name__ field. But FunctionType, GeneratorType, ModuleType etc. are NOT exposed as runtime type objects. Generators are tracked in thread-local GENERATORS map (generator.rs). Need to synthesize Mamba class objects for each type constant. SimpleNamespace can be implemented as a class with dynamic __dict__.

