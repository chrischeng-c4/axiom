# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_open_connection_is_present"
# subject = "asyncio.open_connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.open_connection: api_open_connection_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "open_connection")
print("api_open_connection_is_present OK")
