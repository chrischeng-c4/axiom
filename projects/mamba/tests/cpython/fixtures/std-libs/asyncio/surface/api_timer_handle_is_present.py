# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_timer_handle_is_present"
# subject = "asyncio.TimerHandle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.TimerHandle: api_timer_handle_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "TimerHandle")
print("api_timer_handle_is_present OK")
