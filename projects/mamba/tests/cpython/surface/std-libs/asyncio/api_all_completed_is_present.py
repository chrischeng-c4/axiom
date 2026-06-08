# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_all_completed_is_present"
# subject = "asyncio.ALL_COMPLETED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.ALL_COMPLETED: api_all_completed_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "ALL_COMPLETED")
print("api_all_completed_is_present OK")
