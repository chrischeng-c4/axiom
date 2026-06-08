# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "cancelled_error_is_attr"
# subject = "asyncio.CancelledError"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.CancelledError attribute path not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.CancelledError: cancelled_error_is_attr (surface)."""
import asyncio

assert hasattr(asyncio.CancelledError, "args")
print("cancelled_error_is_attr OK")
