# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_datagram_transport_is_present"
# subject = "asyncio.DatagramTransport"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.DatagramTransport: api_datagram_transport_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "DatagramTransport")
print("api_datagram_transport_is_present OK")
