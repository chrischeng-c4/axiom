# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_server_is_present"
# subject = "asyncio.Server"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Server: api_server_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Server")
print("api_server_is_present OK")
