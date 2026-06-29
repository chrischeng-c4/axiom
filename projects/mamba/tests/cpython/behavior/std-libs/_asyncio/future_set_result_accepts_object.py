# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "behavior"
# case = "future_set_result_accepts_object"
# subject = "_asyncio.Future.set_result(result)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_futures.py"
# status = "filled"
# ///
"""_asyncio.Future.set_result accepts and returns an arbitrary result object."""

from _asyncio import Future


class _W:
    pass


token = _W()
fut = Future()
assert fut.set_result(token) is None
assert fut.done() is True
assert fut.result() is token
print("future_set_result_accepts_object OK")
