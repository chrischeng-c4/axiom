# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_barrier_is_present"
# subject = "asyncio.Barrier"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Barrier: api_barrier_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Barrier")
print("api_barrier_is_present OK")
