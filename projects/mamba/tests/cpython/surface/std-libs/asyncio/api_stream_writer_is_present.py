# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_stream_writer_is_present"
# subject = "asyncio.StreamWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.StreamWriter: api_stream_writer_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "StreamWriter")
print("api_stream_writer_is_present OK")
