# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "map_propagates_task_exception"
# subject = "concurrent.futures.Executor.map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Executor.map: if any mapped task raises, iterating the Executor.map result propagates that exception (ValueError) to the consumer"""
import concurrent.futures


def raise_on_2(x):
    if x == 2:
        raise ValueError(f"bad: {x}")
    return x


with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    raised = False
    try:
        list(ex.map(raise_on_2, [0, 1, 2, 3], timeout=5))
    except ValueError:
        raised = True
assert raised, "Executor.map propagates the task's ValueError to the consumer"

print("map_propagates_task_exception OK")
