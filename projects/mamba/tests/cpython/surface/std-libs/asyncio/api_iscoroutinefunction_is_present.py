# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_iscoroutinefunction_is_present"
# subject = "asyncio.iscoroutinefunction"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.iscoroutinefunction: api_iscoroutinefunction_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "iscoroutinefunction")
print("api_iscoroutinefunction_is_present OK")
