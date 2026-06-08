# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_semaphore_is_present"
# subject = "asyncio.Semaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Semaphore: api_semaphore_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Semaphore")
print("api_semaphore_is_present OK")
