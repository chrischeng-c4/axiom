# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_wait_is_present"
# subject = "asyncio.wait"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.wait: api_wait_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "wait")
print("api_wait_is_present OK")
