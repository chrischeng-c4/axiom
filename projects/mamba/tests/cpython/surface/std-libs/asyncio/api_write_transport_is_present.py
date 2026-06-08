# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_write_transport_is_present"
# subject = "asyncio.WriteTransport"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.WriteTransport: api_write_transport_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "WriteTransport")
print("api_write_transport_is_present OK")
