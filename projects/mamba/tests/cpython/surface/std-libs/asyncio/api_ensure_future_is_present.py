# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_ensure_future_is_present"
# subject = "asyncio.ensure_future"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.ensure_future: api_ensure_future_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "ensure_future")
print("api_ensure_future_is_present OK")
