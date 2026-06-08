# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "submit_returns_future_with_result"
# subject = "concurrent.futures.ThreadPoolExecutor.submit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor.submit: ThreadPoolExecutor.submit returns a concurrent.futures.Future whose .result() yields the task's return value (submit(lambda: 42).result() == 42)"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
    fut = ex.submit(lambda: 42)
    assert isinstance(fut, concurrent.futures.Future), f"submit returns a Future, got {type(fut)!r}"
    result = fut.result(timeout=5)
    assert result == 42, f"future result = {result!r}"

print("submit_returns_future_with_result OK")
