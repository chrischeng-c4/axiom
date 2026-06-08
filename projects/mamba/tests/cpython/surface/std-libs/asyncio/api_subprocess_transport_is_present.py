# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_subprocess_transport_is_present"
# subject = "asyncio.SubprocessTransport"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.SubprocessTransport: api_subprocess_transport_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "SubprocessTransport")
print("api_subprocess_transport_is_present OK")
