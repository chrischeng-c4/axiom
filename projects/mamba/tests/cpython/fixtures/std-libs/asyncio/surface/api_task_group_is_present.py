# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_task_group_is_present"
# subject = "asyncio.TaskGroup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.TaskGroup: api_task_group_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "TaskGroup")
print("api_task_group_is_present OK")
