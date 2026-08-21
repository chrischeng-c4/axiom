# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "queue_is_callable"
# subject = "asyncio.Queue"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.Queue not implemented (resolves non-callable) (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Queue: queue_is_callable (surface)."""
import asyncio

assert callable(asyncio.Queue)
print("queue_is_callable OK")
