# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_create_task_is_present"
# subject = "asyncio.create_task"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.create_task: api_create_task_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "create_task")
print("api_create_task_is_present OK")
