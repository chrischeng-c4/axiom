# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_condition_is_present"
# subject = "asyncio.Condition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Condition: api_condition_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Condition")
print("api_condition_is_present OK")
