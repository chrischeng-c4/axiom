# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "exception_captures_task_error"
# subject = "concurrent.futures.Future.exception"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.exception: Future.exception() returns the exception object raised inside the task (a ValueError with its original message) rather than re-raising it"""
import concurrent.futures


def raises():
    raise ValueError("test error")


with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(raises)
    exc = fut.exception(timeout=5)
    assert isinstance(exc, ValueError), f"captured exception type = {type(exc)!r}"
    assert str(exc) == "test error", f"captured exception msg = {str(exc)!r}"

print("exception_captures_task_error OK")
