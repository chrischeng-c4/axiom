# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_to_thread_is_present"
# subject = "asyncio.to_thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.to_thread: api_to_thread_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "to_thread")
print("api_to_thread_is_present OK")
