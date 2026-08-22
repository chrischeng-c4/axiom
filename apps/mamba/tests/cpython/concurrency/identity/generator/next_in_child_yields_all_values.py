# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "generator"
# dimension = "concurrency"
# case = "next_in_child_yields_all_values"
# subject = "generator.__next__"
# kind = "semantic"
# xfail = "mamba represents a generator as a u64 index into a thread-local registry, so a PARENT-CREATED generator degrades to a bare int in a child thread; this fixture's `extend(gen)` form raises 'int' object is not iterable (a `next()` call on the same object reports 'int' object is not an iterator) (#2846)"
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Concurrency contract: a generator is iterable from a thread that did not create it.

One worker drains a generator built on the main thread. There is a single
consumer, so no interleaving question arises -- this is purely about whether the
generator object survives being handed across a thread boundary. Deterministic.

mamba represents a generator as a `u64` handle into a `thread_local!` registry;
the worker misses the registry and the handle degrades to a bare int. The exact
message tracks the call shape -- `extend(gen)` here reports `'int' object is not
iterable`, while `next(gen)` on the same object reports `'int' object is not an
iterator`. Both are the same defect.
"""
import threading


def counter(limit: int):
    for i in range(limit):
        yield i


gen = counter(5)

result: list = []
error: list = []


def worker() -> None:
    try:
        result.extend(gen)
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: generator not iterable in child: {error[0]}")
elif result != [0, 1, 2, 3, 4]:
    print(f"concurrency: FAIL: wrong values from child: {result} expected=[0, 1, 2, 3, 4]")
else:
    print("concurrency: PASS")
