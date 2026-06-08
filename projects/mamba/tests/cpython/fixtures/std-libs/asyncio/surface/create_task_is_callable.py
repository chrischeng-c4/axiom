# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "create_task_is_callable"
# subject = "asyncio.create_task"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.create_task: create_task_is_callable (surface)."""
import asyncio

assert callable(asyncio.create_task)
print("create_task_is_callable OK")
