# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_as_completed_is_present"
# subject = "asyncio.as_completed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.as_completed: api_as_completed_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "as_completed")
print("api_as_completed_is_present OK")
