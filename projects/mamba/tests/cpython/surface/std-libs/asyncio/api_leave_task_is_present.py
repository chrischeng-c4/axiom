# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_leave_task_is_present"
# subject = "asyncio._leave_task"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio._leave_task: api_leave_task_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "_leave_task")
print("api_leave_task_is_present OK")
