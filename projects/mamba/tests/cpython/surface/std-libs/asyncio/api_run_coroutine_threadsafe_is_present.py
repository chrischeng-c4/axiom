# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_run_coroutine_threadsafe_is_present"
# subject = "asyncio.run_coroutine_threadsafe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.run_coroutine_threadsafe: api_run_coroutine_threadsafe_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "run_coroutine_threadsafe")
print("api_run_coroutine_threadsafe_is_present OK")
