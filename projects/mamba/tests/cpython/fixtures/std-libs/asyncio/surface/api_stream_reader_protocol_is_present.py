# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_stream_reader_protocol_is_present"
# subject = "asyncio.StreamReaderProtocol"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.StreamReaderProtocol: api_stream_reader_protocol_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "StreamReaderProtocol")
print("api_stream_reader_protocol_is_present OK")
