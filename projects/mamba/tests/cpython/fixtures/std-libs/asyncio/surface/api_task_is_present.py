# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_task_is_present"
# subject = "asyncio.Task"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Task: api_task_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Task")
print("api_task_is_present OK")
