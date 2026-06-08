# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_wait_for_is_present"
# subject = "asyncio.wait_for"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.wait_for: api_wait_for_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "wait_for")
print("api_wait_for_is_present OK")
