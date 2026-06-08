# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_default_event_loop_policy_is_present"
# subject = "asyncio.DefaultEventLoopPolicy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.DefaultEventLoopPolicy: api_default_event_loop_policy_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "DefaultEventLoopPolicy")
print("api_default_event_loop_policy_is_present OK")
