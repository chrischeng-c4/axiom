# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_stream_reader_is_present"
# subject = "codecs.StreamReader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.StreamReader: api_stream_reader_is_present (surface)."""
import codecs

assert hasattr(codecs, "StreamReader")
print("api_stream_reader_is_present OK")
