# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_transport_is_present"
# subject = "asyncio.Transport"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Transport: api_transport_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Transport")
print("api_transport_is_present OK")
