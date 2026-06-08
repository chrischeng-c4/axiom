# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_invalid_state_error_is_present"
# subject = "asyncio.InvalidStateError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.InvalidStateError: api_invalid_state_error_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "InvalidStateError")
print("api_invalid_state_error_is_present OK")
