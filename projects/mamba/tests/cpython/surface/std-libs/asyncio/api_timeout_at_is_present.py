# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_timeout_at_is_present"
# subject = "asyncio.timeout_at"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.timeout_at: api_timeout_at_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "timeout_at")
print("api_timeout_at_is_present OK")
