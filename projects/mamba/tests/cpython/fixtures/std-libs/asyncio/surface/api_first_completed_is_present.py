# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_first_completed_is_present"
# subject = "asyncio.FIRST_COMPLETED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.FIRST_COMPLETED: api_first_completed_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "FIRST_COMPLETED")
print("api_first_completed_is_present OK")
