# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "primitives"
# lib = "threading"
# dimension = "concurrency"
# case = "barrier_rounds_complete_without_deadlock"
# subject = "threading.Barrier.wait"
# kind = "semantic"
# xfail = "threading.Barrier.wait is not yet reliable in mamba's threaded runtime"
# mem_carveout = ""
# source = "issue #1126 targeted concurrency stress probe"
# status = "filled"
# ///
"""Concurrency primitive: repeated barrier rounds must complete without deadlock.

The fixture uses a small Barrier rendezvous in multiple rounds and fails fast if
any thread remains stuck. This keeps the property deterministic:

* PASS: all rounds complete, every thread advances through every barrier
* FAIL: timeout, broken barrier, or an incomplete round count
"""
import threading

N = 4
ROUNDS = 25
barrier = threading.Barrier(N)
progress_lock = threading.Lock()
completed = 0
errors: list[str] = []


def worker(_tid: int) -> None:
    global completed
    try:
        for _round in range(ROUNDS):
            barrier.wait(timeout=1.0)
            with progress_lock:
                completed += 1
            barrier.wait(timeout=1.0)
    except Exception as exc:  # pragma: no cover - exercised in the failing runtime
        with progress_lock:
            errors.append(type(exc).__name__)


threads = [threading.Thread(target=worker, args=(idx,)) for idx in range(N)]
for thread in threads:
    thread.start()
for thread in threads:
    thread.join(timeout=5.0)

alive = sum(1 for thread in threads if thread.is_alive())
expected = N * ROUNDS
if errors:
    print(f"concurrency: FAIL: barrier raised {errors[0]}")
elif alive:
    print(f"concurrency: FAIL: deadlock, {alive} threads still alive")
elif completed != expected:
    print(f"concurrency: FAIL: completed={completed} expected={expected}")
else:
    print("concurrency: PASS")
