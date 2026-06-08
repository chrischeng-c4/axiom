# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_multi_loop_child_watcher_is_present"
# subject = "asyncio.MultiLoopChildWatcher"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.MultiLoopChildWatcher: api_multi_loop_child_watcher_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "MultiLoopChildWatcher")
print("api_multi_loop_child_watcher_is_present OK")
