# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_fast_child_watcher_is_present"
# subject = "asyncio.FastChildWatcher"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.FastChildWatcher: api_fast_child_watcher_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "FastChildWatcher")
print("api_fast_child_watcher_is_present OK")
