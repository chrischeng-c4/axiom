# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_stream_writer_is_present"
# subject = "codecs.StreamWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.StreamWriter: api_stream_writer_is_present (surface)."""
import codecs

assert hasattr(codecs, "StreamWriter")
print("api_stream_writer_is_present OK")
