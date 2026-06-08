# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_limit_overrun_error_is_present"
# subject = "asyncio.LimitOverrunError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.LimitOverrunError: api_limit_overrun_error_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "LimitOverrunError")
print("api_limit_overrun_error_is_present OK")
