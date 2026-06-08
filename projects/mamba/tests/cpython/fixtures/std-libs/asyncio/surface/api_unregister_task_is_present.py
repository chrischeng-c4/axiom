# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_unregister_task_is_present"
# subject = "asyncio._unregister_task"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio._unregister_task: api_unregister_task_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "_unregister_task")
print("api_unregister_task_is_present OK")
