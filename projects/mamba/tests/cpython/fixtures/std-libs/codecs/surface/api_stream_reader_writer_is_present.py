# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_stream_reader_writer_is_present"
# subject = "codecs.StreamReaderWriter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.StreamReaderWriter: api_stream_reader_writer_is_present (surface)."""
import codecs

assert hasattr(codecs, "StreamReaderWriter")
print("api_stream_reader_writer_is_present OK")
