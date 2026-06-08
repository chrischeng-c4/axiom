# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_lifo_queue_is_present"
# subject = "asyncio.LifoQueue"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.LifoQueue: api_lifo_queue_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "LifoQueue")
print("api_lifo_queue_is_present OK")
