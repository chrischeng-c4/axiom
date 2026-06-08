# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_broken_barrier_error_is_present"
# subject = "asyncio.BrokenBarrierError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.BrokenBarrierError: api_broken_barrier_error_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "BrokenBarrierError")
print("api_broken_barrier_error_is_present OK")
