# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_cancelled_error_is_present"
# subject = "asyncio.CancelledError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.CancelledError: api_cancelled_error_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "CancelledError")
print("api_cancelled_error_is_present OK")
