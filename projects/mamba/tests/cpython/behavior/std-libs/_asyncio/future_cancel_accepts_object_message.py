# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "behavior"
# case = "future_cancel_accepts_object_message"
# subject = "_asyncio.Future.cancel(msg)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_futures.py"
# status = "filled"
# ///
"""_asyncio.Future.cancel accepts an arbitrary object message."""

from _asyncio import Future


class _W:
    pass


fut = Future()
assert fut.cancel(_W()) is True
assert fut.cancelled() is True
print("future_cancel_accepts_object_message OK")
