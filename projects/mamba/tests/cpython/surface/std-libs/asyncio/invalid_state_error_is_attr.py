# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "invalid_state_error_is_attr"
# subject = "asyncio.InvalidStateError"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.InvalidStateError not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.InvalidStateError: invalid_state_error_is_attr (surface)."""
import asyncio

assert hasattr(asyncio.InvalidStateError, "args")
print("invalid_state_error_is_attr OK")
