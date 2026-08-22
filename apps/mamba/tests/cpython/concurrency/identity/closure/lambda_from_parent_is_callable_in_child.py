# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "closure"
# dimension = "concurrency"
# case = "lambda_from_parent_is_callable_in_child"
# subject = "lambda"
# kind = "semantic"
# xfail = "mamba represents a closure as a u64 index into a thread-local registry, so a parent-created lambda resolves to a bare int in a child thread (#2844)"
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Concurrency contract: a function object is not owned by the thread that made it.

A callable created on the main thread and invoked from a worker must still be
callable. This is an object-identity property, not a race: it is deterministic
and fails identically on every run, so the verdict needs no repetition.

mamba represents a closure as a `u64` handle indexing a `thread_local!`
registry. A worker thread misses that registry and the handle surfaces as the
bare integer it always was, giving `'int' object is not callable`. This is the
ABSOLUTE clause of the facet contract -- an impossible value, not a permitted
race.
"""
import threading

double = lambda v: v * 2

result: list = []
error: list = []


def worker() -> None:
    try:
        result.append(double(21))
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: parent lambda not callable in child: {error[0]}")
elif result != [42]:
    print(f"concurrency: FAIL: wrong value from parent lambda: {result} expected=[42]")
else:
    print("concurrency: PASS")
