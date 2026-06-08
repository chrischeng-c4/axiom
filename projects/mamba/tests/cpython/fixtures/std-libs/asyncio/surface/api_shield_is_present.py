# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_shield_is_present"
# subject = "asyncio.shield"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.shield: api_shield_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "shield")
print("api_shield_is_present OK")
