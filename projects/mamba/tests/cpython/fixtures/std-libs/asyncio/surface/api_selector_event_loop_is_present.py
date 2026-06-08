# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_selector_event_loop_is_present"
# subject = "asyncio.SelectorEventLoop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.SelectorEventLoop: api_selector_event_loop_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "SelectorEventLoop")
print("api_selector_event_loop_is_present OK")
