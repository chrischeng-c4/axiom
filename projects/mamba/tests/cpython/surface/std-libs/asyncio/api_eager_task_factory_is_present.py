# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_eager_task_factory_is_present"
# subject = "asyncio.eager_task_factory"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.eager_task_factory: api_eager_task_factory_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "eager_task_factory")
print("api_eager_task_factory_is_present OK")
