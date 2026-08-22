# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "random"
# dimension = "concurrency"
# case = "child_built_random_instance_draws_in_child"
# subject = "random.Random"
# kind = "semantic"
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Control for `parent_random_instance_draws_in_child`: instance built in the worker.

Expected green today and after the ownership fix. Like its sibling it asserts
only shape, not the drawn value, so that #3084's seeding divergence cannot make
this control red for an unrelated reason.
"""
import random
import threading

result: list = []
error: list = []


def worker() -> None:
    try:
        result.append(random.Random(1234).random())
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: child-built Random not usable: {error[0]}")
elif not result:
    print("concurrency: FAIL: worker produced no draw and raised nothing")
elif not isinstance(result[0], float) or not (0.0 <= result[0] < 1.0):
    print(f"concurrency: FAIL: draw not a float in [0,1): {result[0]!r}")
else:
    print("concurrency: PASS")
