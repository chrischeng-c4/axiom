# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_getincrementaldecoder_is_present"
# subject = "codecs.getincrementaldecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.getincrementaldecoder: api_getincrementaldecoder_is_present (surface)."""
import codecs

assert hasattr(codecs, "getincrementaldecoder")
print("api_getincrementaldecoder_is_present OK")
