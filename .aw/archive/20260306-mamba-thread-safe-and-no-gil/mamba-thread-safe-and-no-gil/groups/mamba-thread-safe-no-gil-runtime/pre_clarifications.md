---
change: mamba-thread-safe-and-no-gil
group: mamba-thread-safe-no-gil-runtime
date: 2026-03-06
status: answered
---

# Pre-Clarifications

### Q1: Locking Granularity
- **Answer**: Per-object locks. Use RwLock per collection object (List, Dict, Set). Standard no-GIL approach with better concurrency.

### Q2: GC Design
- **Answer**: Stop-the-world approach for mark phase. Simpler implementation, pause all threads during mark. Good enough for most workloads.

### Q3: Python 3.13 Compatibility
- **Answer**: Mamba runtime only. Thread-safety for Mamba's own execution model. No need to align with CPython 3.13t free-threaded build.

### Q4: Performance Trade-offs
- **Answer**: No specific benchmarks to maintain. Start simple with straightforward atomics/locks, optimize later.

### Q5: Atomic Refcounting Optimization
- **Answer**: Start simple, optimize later. Get basic thread-safety working first with straightforward atomics/locks. Add biased locking / immortal objects optimizations in a follow-up.

