# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_run_is_present"
# subject = "asyncio.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.run: api_run_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "run")
print("api_run_is_present OK")
