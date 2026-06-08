# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_threaded_child_watcher_is_present"
# subject = "asyncio.ThreadedChildWatcher"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.ThreadedChildWatcher: api_threaded_child_watcher_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "ThreadedChildWatcher")
print("api_threaded_child_watcher_is_present OK")
