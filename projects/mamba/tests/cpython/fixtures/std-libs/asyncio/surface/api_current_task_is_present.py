# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_current_task_is_present"
# subject = "asyncio.current_task"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.current_task: api_current_task_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "current_task")
print("api_current_task_is_present OK")
