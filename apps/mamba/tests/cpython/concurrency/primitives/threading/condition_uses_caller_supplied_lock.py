# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "condition_uses_caller_supplied_lock"
# subject = "threading.Condition"
# kind = "semantic"
# xfail = "issue #3128: Condition(lock) validates the lock and then discards it"
# mem_carveout = ""
# source = "issue #3128 negative-control probe"
# status = "filled"
# ///
"""Concurrency primitive: Condition(lock) shares the lock the caller passed.

CPython's `Condition(lock)` adopts that exact lock object, so entering the
condition is the same mutual exclusion as entering the lock. A Condition that
silently substitutes a private lock still runs, still returns, and protects
nothing the caller thinks it protects.

* PASS: acquiring the condition locks the caller's lock and releasing frees it
* FAIL: the caller's lock is untouched
"""
import threading

problems: list[str] = []

lock = threading.Lock()
condition = threading.Condition(lock)

if lock.locked():
    problems.append("caller lock was already held before acquire()")

condition.acquire()
held_during = lock.locked()
condition.release()
held_after = lock.locked()

if held_during is not True:
    problems.append(f"lock.locked() was {held_during!r} inside the condition, expected True")
if held_after is not False:
    problems.append(f"lock.locked() was {held_after!r} after release, expected False")

# the `with` form must adopt the same lock
with condition:
    held_in_with = lock.locked()
held_out_of_with = lock.locked()

if held_in_with is not True:
    problems.append(f"lock.locked() was {held_in_with!r} inside `with condition:`, expected True")
if held_out_of_with is not False:
    problems.append(f"lock.locked() was {held_out_of_with!r} after `with condition:`, expected False")

if problems:
    print(f"concurrency: FAIL: {problems[0]}")
else:
    print("concurrency: PASS")
