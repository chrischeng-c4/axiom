# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_base_protocol_is_present"
# subject = "asyncio.BaseProtocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.BaseProtocol: api_base_protocol_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "BaseProtocol")
print("api_base_protocol_is_present OK")
