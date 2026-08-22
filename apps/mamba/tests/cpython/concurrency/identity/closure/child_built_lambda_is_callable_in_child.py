# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "closure"
# dimension = "concurrency"
# case = "child_built_lambda_is_callable_in_child"
# subject = "lambda"
# kind = "semantic"
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Control for `lambda_from_parent_is_callable_in_child`: lambda built in the worker.

Expected green today and after the ownership fix. Proves the sibling xfail
indicts the thread boundary rather than closures, and guards the fix against
repairing parent-created callables at the cost of child-created ones.
"""
import threading

result: list = []
error: list = []


def worker() -> None:
    try:
        double = lambda v: v * 2
        result.append(double(21))
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: child-built lambda not callable: {error[0]}")
elif result != [42]:
    print(f"concurrency: FAIL: wrong value from child-built lambda: {result} expected=[42]")
else:
    print("concurrency: PASS")
