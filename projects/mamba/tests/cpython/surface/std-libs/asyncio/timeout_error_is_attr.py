# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "timeout_error_is_attr"
# subject = "asyncio.TimeoutError"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.TimeoutError attribute path not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.TimeoutError: timeout_error_is_attr (surface)."""
import asyncio

assert hasattr(asyncio.TimeoutError, "args")
print("timeout_error_is_attr OK")
