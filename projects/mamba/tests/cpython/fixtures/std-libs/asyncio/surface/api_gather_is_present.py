# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_gather_is_present"
# subject = "asyncio.gather"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.gather: api_gather_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "gather")
print("api_gather_is_present OK")
