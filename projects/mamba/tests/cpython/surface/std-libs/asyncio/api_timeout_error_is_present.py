# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_timeout_error_is_present"
# subject = "asyncio.TimeoutError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.TimeoutError: api_timeout_error_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "TimeoutError")
print("api_timeout_error_is_present OK")
