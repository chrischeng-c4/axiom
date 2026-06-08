# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_runner_is_present"
# subject = "asyncio.Runner"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Runner: api_runner_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Runner")
print("api_runner_is_present OK")
