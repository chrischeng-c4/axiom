# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "errors"
# case = "cancel_after_done_returns_false"
# subject = "concurrent.futures.Future.cancel"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.Future.cancel: calling Future.cancel() after a result has been set returns False and does NOT raise (the completed future is uncancellable)"""
import concurrent.futures

f = concurrent.futures.Future()
f.set_result(1)
assert f.done(), "future is done after set_result"
# A completed future cannot be cancelled: cancel() returns False, no raise.
assert f.cancel() is False, "cancel() on a done future returns False"
assert f.cancelled() is False, "a done future is not cancelled"
assert f.result() == 1, "result is still readable after the failed cancel"

print("cancel_after_done_returns_false OK")
