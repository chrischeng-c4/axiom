# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_buffered_protocol_is_present"
# subject = "asyncio.BufferedProtocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.BufferedProtocol: api_buffered_protocol_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "BufferedProtocol")
print("api_buffered_protocol_is_present OK")
