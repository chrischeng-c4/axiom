# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_bounded_semaphore_is_present"
# subject = "asyncio.BoundedSemaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.BoundedSemaphore: api_bounded_semaphore_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "BoundedSemaphore")
print("api_bounded_semaphore_is_present OK")
