# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_all_tasks_is_present"
# subject = "asyncio.all_tasks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.all_tasks: api_all_tasks_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "all_tasks")
print("api_all_tasks_is_present OK")
