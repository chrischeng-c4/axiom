# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_getdecoder_is_present"
# subject = "codecs.getdecoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.getdecoder: api_getdecoder_is_present (surface)."""
import codecs

assert hasattr(codecs, "getdecoder")
print("api_getdecoder_is_present OK")
