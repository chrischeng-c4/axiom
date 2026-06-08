# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_queue_is_present"
# subject = "asyncio.Queue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Queue: api_queue_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Queue")
print("api_queue_is_present OK")
