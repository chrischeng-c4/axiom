# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_base_transport_is_present"
# subject = "asyncio.BaseTransport"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.BaseTransport: api_base_transport_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "BaseTransport")
print("api_base_transport_is_present OK")
