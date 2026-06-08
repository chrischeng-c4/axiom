# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_timeout_is_present"
# subject = "asyncio.Timeout"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Timeout: api_timeout_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Timeout")
print("api_timeout_is_present OK")
