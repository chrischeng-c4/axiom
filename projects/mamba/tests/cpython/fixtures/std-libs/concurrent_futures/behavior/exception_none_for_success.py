# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "exception_none_for_success"
# subject = "concurrent.futures.Future.exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.exception: Future.exception() returns None for a future that completed successfully"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(lambda: 99)
    assert fut.result(timeout=5) == 99, "task succeeds"
    assert fut.exception(timeout=5) is None, "exception() is None for a successful future"

print("exception_none_for_success OK")
