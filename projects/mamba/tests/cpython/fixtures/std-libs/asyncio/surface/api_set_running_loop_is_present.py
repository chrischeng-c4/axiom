# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_set_running_loop_is_present"
# subject = "asyncio._set_running_loop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio._set_running_loop: api_set_running_loop_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "_set_running_loop")
print("api_set_running_loop_is_present OK")
