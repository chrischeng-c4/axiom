# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_start_server_is_present"
# subject = "asyncio.start_server"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.start_server: api_start_server_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "start_server")
print("api_start_server_is_present OK")
