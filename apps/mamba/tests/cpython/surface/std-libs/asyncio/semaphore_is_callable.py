# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "semaphore_is_callable"
# subject = "asyncio.Semaphore"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.Semaphore not implemented (resolves non-callable) (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Semaphore: semaphore_is_callable (surface)."""
import asyncio

assert callable(asyncio.Semaphore)
print("semaphore_is_callable OK")
