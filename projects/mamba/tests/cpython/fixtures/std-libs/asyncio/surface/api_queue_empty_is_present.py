# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_queue_empty_is_present"
# subject = "asyncio.QueueEmpty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.QueueEmpty: api_queue_empty_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "QueueEmpty")
print("api_queue_empty_is_present OK")
