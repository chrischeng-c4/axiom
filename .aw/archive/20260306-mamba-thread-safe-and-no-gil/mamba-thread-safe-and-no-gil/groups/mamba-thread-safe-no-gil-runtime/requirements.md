---
change: mamba-thread-safe-and-no-gil
group: mamba-thread-safe-no-gil-runtime
date: 2026-03-06
---

# Requirements

Make the cclab-mamba runtime thread-safe to support execution in a no-GIL (Global Interpreter Lock) environment. Key requirements include:
1. Transition MbObject reference counting from non-atomic u32 to atomic (AtomicU32/AtomicUsize) using Acquire/Release semantics.
2. Redesign the GcState and ROOTS management to be global and thread-safe, moving away from thread_local storage.
3. Implement synchronization for core collection types (List, Dict, Set) within ObjData to prevent data races during concurrent mutation.
4. Ensure the runtime can safely share and manage objects across multiple threads without relying on an external lock.
5. Verify thread-safety with concurrent stress tests and race detection.
