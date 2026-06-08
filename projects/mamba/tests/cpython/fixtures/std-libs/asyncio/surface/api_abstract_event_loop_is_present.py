# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_abstract_event_loop_is_present"
# subject = "asyncio.AbstractEventLoop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.AbstractEventLoop: api_abstract_event_loop_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "AbstractEventLoop")
print("api_abstract_event_loop_is_present OK")
