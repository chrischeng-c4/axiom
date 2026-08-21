# Operational AssertionPass seed for the `tracemalloc` module — the
# CPython runtime memory-allocation tracer used by every profiler /
# leak detector / pytest-memory plugin / deepcopy benchmarker. No
# fixture coverage yet for tracemalloc.
#
# The matching subset between mamba and CPython is the lifecycle-and-
# shape contract: tracing state transitions (`is_tracing()` flips
# False → True → False around `start()` / `stop()`), the tuple shape
# of `get_traced_memory()` (current, peak as a 2-tuple of ints), the
# fact that `take_snapshot()` returns a non-None object, and the
# callable identity of every top-level entry point (`start` / `stop` /
# `is_tracing` / `get_traced_memory` / `take_snapshot` / `clear_traces`
# / `reset_peak` / `get_object_traceback`).
#
# Surface in this fixture:
#   • tracemalloc.is_tracing() — boolean lifecycle gate;
#       — False before start(), True between start() and stop(), False
#         again after stop();
#       — re-entrant start()/stop() round-trips return the gate to its
#         original value;
#   • tracemalloc.get_traced_memory() — (current_bytes, peak_bytes)
#     tuple of two ints, non-negative;
#   • tracemalloc.take_snapshot() — returns a non-None snapshot value;
#   • tracemalloc.clear_traces() — callable from active state;
#   • tracemalloc.reset_peak() — callable from active state;
#   • module-level callables — start / stop / is_tracing /
#     get_traced_memory / take_snapshot / clear_traces / reset_peak /
#     get_object_traceback all `callable(...)` True.
#
# Behavioral edges that DIVERGE on mamba (timeit.default_timer
# returning int rather than float, timeit.default_number /
# default_repeat sentinels, timeit.template as str, timeit.timeit /
# Timer instance method returning a measurable float, timeit.repeat
# yielding a list of repeat-many floats, timeit.Timer being a class
# with print_exc, tracemalloc.get_tracemalloc_memory() / Filter /
# Snapshot class identity, take_snapshot() returning a Snapshot
# instance) are covered in
# `lang_timeit_default_number_timer_tracemalloc_silent.py`.
import tracemalloc
from typing import Any

_ledger: list[int] = []

# 1) Lifecycle: not tracing at module entry
assert tracemalloc.is_tracing() == False; _ledger.append(1)

# 2) start() flips the gate to True
tracemalloc.start()
assert tracemalloc.is_tracing() == True; _ledger.append(1)

# 3) get_traced_memory shape — (current, peak) as a 2-tuple
_mem: Any = tracemalloc.get_traced_memory()
assert isinstance(_mem, tuple); _ledger.append(1)
assert len(_mem) == 2; _ledger.append(1)
assert isinstance(_mem[0], int); _ledger.append(1)
assert isinstance(_mem[1], int); _ledger.append(1)
# Both counters are non-negative
assert _mem[0] >= 0; _ledger.append(1)
assert _mem[1] >= 0; _ledger.append(1)
# Peak >= current at any moment
assert _mem[1] >= _mem[0]; _ledger.append(1)

# 4) take_snapshot returns a non-None value
_snap: Any = tracemalloc.take_snapshot()
assert _snap is not None; _ledger.append(1)

# 5) clear_traces / reset_peak are no-arg operations
tracemalloc.clear_traces()
_ledger.append(1)
tracemalloc.reset_peak()
_ledger.append(1)

# After clear, current count should be 0 or low
_mem2: Any = tracemalloc.get_traced_memory()
assert isinstance(_mem2, tuple); _ledger.append(1)
assert len(_mem2) == 2; _ledger.append(1)

# 6) stop() flips the gate back to False
tracemalloc.stop()
assert tracemalloc.is_tracing() == False; _ledger.append(1)

# 7) Lifecycle is re-entrant — start() again then stop() again
tracemalloc.start()
assert tracemalloc.is_tracing() == True; _ledger.append(1)
tracemalloc.stop()
assert tracemalloc.is_tracing() == False; _ledger.append(1)

# 8) Module-level callables — every documented public entry
assert callable(tracemalloc.start); _ledger.append(1)
assert callable(tracemalloc.stop); _ledger.append(1)
assert callable(tracemalloc.is_tracing); _ledger.append(1)
assert callable(tracemalloc.get_traced_memory); _ledger.append(1)
assert callable(tracemalloc.take_snapshot); _ledger.append(1)
assert callable(tracemalloc.clear_traces); _ledger.append(1)
assert callable(tracemalloc.reset_peak); _ledger.append(1)
assert callable(tracemalloc.get_object_traceback); _ledger.append(1)

# NB: tracemalloc.get_tracemalloc_memory() (the bookkeeping-overhead
# counter), tracemalloc.Filter / Snapshot class identity, and the
# Snapshot-typed return of take_snapshot() DIVERGE on mamba and are
# pinned in the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_tracemalloc_start_stop_snapshot_ops {sum(_ledger)} asserts")
