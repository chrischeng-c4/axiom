# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "random"
# dimension = "concurrency"
# case = "parent_random_instance_draws_in_child"
# subject = "random.Random"
# kind = "semantic"
# xfail = "mamba keys random.Random state by a u64 index into a thread-local registry; an instance CREATED IN THE PARENT misses the child's registry and .random resolves to None, giving 'NoneType' object is not callable (#2968). An instance created inside the child works, so the defect is the parent-to-child handoff."
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Concurrency contract: a Random instance draws from a thread that did not make it.

Paired with `child_built_random_instance_draws_in_child.py` to separate "random
is broken under threads" from "the handoff is broken".

The verdict deliberately checks only that the draw happened and is a float in
[0, 1). It does NOT pin the value, because mamba's seeded stream disagrees with
CPython's for an unrelated reason (#3084, numpy-style MT seeding). Pinning the
value here would make this fixture fail for two independent causes and stay red
after the ownership fix lands, which would make it useless as a gate for that
fix.
"""
import random
import threading

rng = random.Random(1234)

result: list = []
error: list = []


def worker() -> None:
    try:
        result.append(rng.random())
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: parent Random not usable in child: {error[0]}")
elif not result:
    print("concurrency: FAIL: worker produced no draw and raised nothing")
elif not isinstance(result[0], float) or not (0.0 <= result[0] < 1.0):
    print(f"concurrency: FAIL: draw not a float in [0,1): {result[0]!r}")
else:
    print("concurrency: PASS")
