# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_pidfd_child_watcher_is_present"
# subject = "asyncio.PidfdChildWatcher"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.PidfdChildWatcher: api_pidfd_child_watcher_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "PidfdChildWatcher")
print("api_pidfd_child_watcher_is_present OK")
