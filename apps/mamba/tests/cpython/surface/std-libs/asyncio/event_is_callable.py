# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "event_is_callable"
# subject = "asyncio.Event"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.Event not implemented (resolves non-callable) (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Event: event_is_callable (surface)."""
import asyncio

assert callable(asyncio.Event)
print("event_is_callable OK")
