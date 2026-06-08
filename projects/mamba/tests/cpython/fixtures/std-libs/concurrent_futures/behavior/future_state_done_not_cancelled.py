# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "behavior"
# case = "future_state_done_not_cancelled"
# subject = "concurrent.futures.Future.done"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.done: after .result() returns, a successful future reports done() True and cancelled() False"""
import concurrent.futures

with concurrent.futures.ThreadPoolExecutor(max_workers=1) as ex:
    fut = ex.submit(lambda: "hello")
    val = fut.result(timeout=5)
    assert val == "hello", f"future value = {val!r}"
    assert fut.done() is True, "future is done after result"
    assert fut.cancelled() is False, "successful future is not cancelled"

print("future_state_done_not_cancelled OK")
