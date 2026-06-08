# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_lock_is_present"
# subject = "asyncio.Lock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Lock: api_lock_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Lock")
print("api_lock_is_present OK")
