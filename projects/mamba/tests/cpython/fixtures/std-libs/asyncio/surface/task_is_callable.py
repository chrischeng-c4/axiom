# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "task_is_callable"
# subject = "asyncio.Task"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.Task not implemented (resolves non-callable) (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Task: task_is_callable (surface)."""
import asyncio

assert callable(asyncio.Task)
print("task_is_callable OK")
