# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "lock_is_callable"
# subject = "asyncio.Lock"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.Lock not implemented (resolves non-callable) (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Lock: lock_is_callable (surface)."""
import asyncio

assert callable(asyncio.Lock)
print("lock_is_callable OK")
