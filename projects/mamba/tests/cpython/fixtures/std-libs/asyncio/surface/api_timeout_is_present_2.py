# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_timeout_is_present_2"
# subject = "asyncio.timeout"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.timeout: api_timeout_is_present_2 (surface)."""
import asyncio

assert hasattr(asyncio, "timeout")
print("api_timeout_is_present_2 OK")
