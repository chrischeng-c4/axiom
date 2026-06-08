# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "future_is_callable"
# subject = "asyncio.Future"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.Future not implemented (resolves non-callable) (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Future: future_is_callable (surface)."""
import asyncio

assert callable(asyncio.Future)
print("future_is_callable OK")
