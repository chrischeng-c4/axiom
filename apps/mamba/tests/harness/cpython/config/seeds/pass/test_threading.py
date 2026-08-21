# test_threading.py — #3424 axis-1 stdlib threading AssertionPass seed.
#
# Mamba-authored seed exercising the `threading` module surface called
# out in the issue:
#   Thread start/join, Lock/RLock, Event set/wait, local thread-data,
#   current_thread.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. current_thread is the main thread on import; main_thread()
#      identity; get_ident() returns int.
#   3. Thread(target=...) — start + join + return None after join;
#      .name + .is_alive() transitions True→False; sentinel result list
#      reflects target's side effect.
#   4. Lock — acquire / release / context manager.
#   5. RLock — re-entrant acquire by the same thread.
#   6. Event — initial False; set/wait/clear cycle.
#   7. threading.local — attribute storage per-thread.
#   8. enumerate() / active_count() — bookkeeping.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: test_threading N asserts` to stdout.

import threading

_ledger: list[int] = []


# Module-level helpers — no closures (mamba top-level def quirk).
# A target that pushes a sentinel into a shared list. Plain list ops
# are fine here because join() establishes happens-before with the
# spawning thread before we read.
_results: list[int] = []


def _push_42() -> None:
    _results.append(42)


_local_holder = threading.local()


def _set_local_value() -> None:
    _local_holder.value = "thread-set"


# 1. Module identity + public surface.
assert threading.__name__ == "threading", "threading.__name__"
_ledger.append(1)
assert hasattr(threading, "Thread"), "exposes Thread"
_ledger.append(1)
assert hasattr(threading, "Lock"), "exposes Lock"
_ledger.append(1)
assert hasattr(threading, "RLock"), "exposes RLock"
_ledger.append(1)
assert hasattr(threading, "Event"), "exposes Event"
_ledger.append(1)
assert hasattr(threading, "current_thread"), "exposes current_thread"
_ledger.append(1)
assert hasattr(threading, "main_thread"), "exposes main_thread"
_ledger.append(1)
assert hasattr(threading, "local"), "exposes local"
_ledger.append(1)

# 2. current_thread / main_thread / get_ident.
_ct = threading.current_thread()
assert isinstance(_ct, threading.Thread), "current_thread returns Thread instance"
_ledger.append(1)
_mt = threading.main_thread()
assert _mt is _ct, "on module import, current_thread is main_thread"
_ledger.append(1)
_ident = threading.get_ident()
assert isinstance(_ident, int), "get_ident returns int"
_ledger.append(1)
assert _ident > 0, "get_ident > 0"
_ledger.append(1)

# 3. Thread — start / join / side effect.
_t = threading.Thread(target=_push_42, name="worker-1")
assert _t.name == "worker-1", "Thread.name reflects constructor arg"
_ledger.append(1)
assert _t.is_alive() == False, "thread not alive before start"
_ledger.append(1)
_t.start()
_t.join(timeout=5.0)
assert _t.is_alive() == False, "thread not alive after join"
_ledger.append(1)
# Side effect ran exactly once.
assert _results == [42], "target ran, appended 42 to shared list"
_ledger.append(1)

# 4. Lock — acquire / release / context manager.
_lock = threading.Lock()
assert _lock.acquire(timeout=1.0) == True, "Lock.acquire returns True"
_ledger.append(1)
_lock.release()
# Context-manager form.
with _lock:
    _in_ctx = True
assert _in_ctx == True, "Lock context manager entered the body"
_ledger.append(1)

# 5. RLock — re-entrant by the same thread.
_rlock = threading.RLock()
assert _rlock.acquire() == True, "RLock first acquire succeeds"
_ledger.append(1)
assert _rlock.acquire() == True, "RLock second acquire by same thread succeeds (re-entrant)"
_ledger.append(1)
_rlock.release()
_rlock.release()

# 6. Event — initial False; set/wait/clear cycle.
_evt = threading.Event()
assert _evt.is_set() == False, "Event initial state is unset"
_ledger.append(1)
_evt.set()
assert _evt.is_set() == True, "Event.set transitions to set"
_ledger.append(1)
# wait() on a set Event returns True immediately.
assert _evt.wait(timeout=1.0) == True, "Event.wait on set returns True"
_ledger.append(1)
_evt.clear()
assert _evt.is_set() == False, "Event.clear transitions back to unset"
_ledger.append(1)
# wait() with timeout on cleared Event returns False (no signal).
assert _evt.wait(timeout=0.05) == False, "Event.wait with timeout returns False"
_ledger.append(1)

# 7. threading.local — per-thread attribute storage.
# Set on the main thread first.
_local_holder.value = "main-set"
assert _local_holder.value == "main-set", "local attribute readable on main thread"
_ledger.append(1)
# Spawn a thread that mutates the local; main thread's binding stays.
_t2 = threading.Thread(target=_set_local_value, name="local-worker")
_t2.start()
_t2.join(timeout=5.0)
# Other-thread mutation isolated — main thread still sees its own value.
assert _local_holder.value == "main-set", (
    "threading.local isolates attribute across threads"
)
_ledger.append(1)

# 8. enumerate() / active_count() — bookkeeping.
_threads = threading.enumerate()
assert isinstance(_threads, list), "enumerate returns a list"
_ledger.append(1)
assert _mt in _threads, "main thread appears in enumerate()"
_ledger.append(1)
assert threading.active_count() >= 1, "active_count >= 1 (at least the main thread)"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: test_threading {len(_ledger)} asserts")
