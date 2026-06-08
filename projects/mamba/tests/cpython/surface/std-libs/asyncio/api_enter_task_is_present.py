# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_enter_task_is_present"
# subject = "asyncio._enter_task"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio._enter_task: api_enter_task_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "_enter_task")
print("api_enter_task_is_present OK")
