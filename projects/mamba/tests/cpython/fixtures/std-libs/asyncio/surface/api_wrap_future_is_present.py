# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_wrap_future_is_present"
# subject = "asyncio.wrap_future"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.wrap_future: api_wrap_future_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "wrap_future")
print("api_wrap_future_is_present OK")
