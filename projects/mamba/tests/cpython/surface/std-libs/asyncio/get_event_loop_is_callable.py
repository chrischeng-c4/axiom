# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "get_event_loop_is_callable"
# subject = "asyncio.get_event_loop"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.get_event_loop not implemented (resolves non-callable) (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.get_event_loop: get_event_loop_is_callable (surface)."""
import asyncio

assert callable(asyncio.get_event_loop)
print("get_event_loop_is_callable OK")
