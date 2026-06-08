# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_abstract_event_loop_policy_is_present"
# subject = "asyncio.AbstractEventLoopPolicy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.AbstractEventLoopPolicy: api_abstract_event_loop_policy_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "AbstractEventLoopPolicy")
print("api_abstract_event_loop_policy_is_present OK")
