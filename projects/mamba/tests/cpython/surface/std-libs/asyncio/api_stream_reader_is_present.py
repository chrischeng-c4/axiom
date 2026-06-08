# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_stream_reader_is_present"
# subject = "asyncio.StreamReader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.StreamReader: api_stream_reader_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "StreamReader")
print("api_stream_reader_is_present OK")
