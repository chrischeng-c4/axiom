# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_datagram_protocol_is_present"
# subject = "asyncio.DatagramProtocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.DatagramProtocol: api_datagram_protocol_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "DatagramProtocol")
print("api_datagram_protocol_is_present OK")
