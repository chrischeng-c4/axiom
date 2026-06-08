# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_protocol_is_present"
# subject = "asyncio.Protocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Protocol: api_protocol_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Protocol")
print("api_protocol_is_present OK")
