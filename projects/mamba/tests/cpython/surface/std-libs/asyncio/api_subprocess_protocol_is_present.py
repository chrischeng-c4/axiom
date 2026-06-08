# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_subprocess_protocol_is_present"
# subject = "asyncio.SubprocessProtocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.SubprocessProtocol: api_subprocess_protocol_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "SubprocessProtocol")
print("api_subprocess_protocol_is_present OK")
