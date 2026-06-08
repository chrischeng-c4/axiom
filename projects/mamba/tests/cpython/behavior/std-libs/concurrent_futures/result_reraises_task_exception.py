# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "result_reraises_task_exception"
# subject = "concurrent.futures.Future.result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.result: an exception raised inside a task is re-raised (same type and message) when the future's .result() is called"""
import concurrent.futures


def bad():
    raise RuntimeError("task failed")


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(bad)

raised = False
try:
    fut.result(timeout=5)
except RuntimeError as e:
    raised = True
    assert str(e) == "task failed", f"re-raised message = {str(e)!r}"
assert raised, "RuntimeError re-raised from future.result()"

print("result_reraises_task_exception OK")
