# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_threading_lock_event_ops"
# subject = "cpython321.test_threading_lock_event_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_threading_lock_event_ops.py"
# status = "filled"
# ///
"""cpython321.test_threading_lock_event_ops: execute CPython 3.12 seed test_threading_lock_event_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for threading primitives: current
# thread introspection, Lock/RLock acquire-release, Event set/clear/is_set,
# Condition context-manager protocol. Surface: `threading.current_thread()`
# returns a Thread object whose `.name` attribute is non-None; module-level
# `active_count()` reports at least one running thread (the main thread).
# `threading.Lock()` and `threading.RLock()` both yield primitives whose
# `acquire()` returns True on success; non-reentrant Lock supports the
# context-manager protocol so `with lock:` enters/exits cleanly.
# `threading.Event()` constructs an unset flag; `is_set()` reflects
# current state, `set()` transitions to True, and `clear()` transitions
# back to False. `threading.Condition()` supports the context-manager
# protocol via its associated lock.
import threading
_ledger: list[int] = []

# current_thread returns a Thread object with a name attribute
t = threading.current_thread()
assert t is not None; _ledger.append(1)
assert hasattr(t, "name"); _ledger.append(1)

# active_count is >= 1 (main thread always alive)
assert threading.active_count() >= 1; _ledger.append(1)

# Lock acquire/release
lock = threading.Lock()
assert lock.acquire() == True; _ledger.append(1)
lock.release()

# Lock works as a context manager
with lock:
    assert True; _ledger.append(1)

# RLock acquire/release
rl = threading.RLock()
assert rl.acquire() == True; _ledger.append(1)
rl.release()

# Event initially unset
e = threading.Event()
assert e.is_set() == False; _ledger.append(1)
e.set()
assert e.is_set() == True; _ledger.append(1)
e.clear()
assert e.is_set() == False; _ledger.append(1)

# Condition works as a context manager
cv = threading.Condition()
with cv:
    assert True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_threading_lock_event_ops {sum(_ledger)} asserts")
