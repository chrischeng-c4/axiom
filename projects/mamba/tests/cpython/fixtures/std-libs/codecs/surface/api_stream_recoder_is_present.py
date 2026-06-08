# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_stream_recoder_is_present"
# subject = "codecs.StreamRecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.StreamRecoder: api_stream_recoder_is_present (surface)."""
import codecs

assert hasattr(codecs, "StreamRecoder")
print("api_stream_recoder_is_present OK")
