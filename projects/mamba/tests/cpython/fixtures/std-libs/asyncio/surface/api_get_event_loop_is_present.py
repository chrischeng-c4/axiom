# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_get_event_loop_is_present"
# subject = "asyncio.get_event_loop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.get_event_loop: api_get_event_loop_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "get_event_loop")
print("api_get_event_loop_is_present OK")
