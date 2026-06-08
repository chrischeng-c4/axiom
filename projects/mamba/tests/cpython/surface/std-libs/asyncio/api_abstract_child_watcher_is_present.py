# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_abstract_child_watcher_is_present"
# subject = "asyncio.AbstractChildWatcher"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.AbstractChildWatcher: api_abstract_child_watcher_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "AbstractChildWatcher")
print("api_abstract_child_watcher_is_present OK")
