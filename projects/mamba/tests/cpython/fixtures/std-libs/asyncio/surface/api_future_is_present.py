# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_future_is_present"
# subject = "asyncio.Future"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Future: api_future_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Future")
print("api_future_is_present OK")
