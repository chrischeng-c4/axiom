# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "identity"
# lib = "generator"
# dimension = "concurrency"
# case = "child_built_generator_yields_all_values"
# subject = "generator.__next__"
# kind = "semantic"
# mem_carveout = ""
# source = "#2968 stage-1 ownership probe matrix"
# status = "filled"
# ///
"""Control for `next_in_child_yields_all_values`: generator created in the worker.

Expected green today and after the ownership fix. The generator function is
defined at module scope -- only the generator *object* is created in the child,
which is what isolates the handle handoff from the function definition itself.
"""
import threading


def counter(limit: int):
    for i in range(limit):
        yield i


result: list = []
error: list = []


def worker() -> None:
    try:
        result.extend(counter(5))
    except BaseException as exc:
        error.append(f"{type(exc).__name__}: {exc}")


t = threading.Thread(target=worker)
t.start()
t.join()

if error:
    print(f"concurrency: FAIL: child-built generator not iterable: {error[0]}")
elif result != [0, 1, 2, 3, 4]:
    print(f"concurrency: FAIL: wrong values from child-built generator: {result} expected=[0, 1, 2, 3, 4]")
else:
    print("concurrency: PASS")
