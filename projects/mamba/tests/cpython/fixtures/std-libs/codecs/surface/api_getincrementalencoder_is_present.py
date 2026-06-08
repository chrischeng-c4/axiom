# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_getincrementalencoder_is_present"
# subject = "codecs.getincrementalencoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.getincrementalencoder: api_getincrementalencoder_is_present (surface)."""
import codecs

assert hasattr(codecs, "getincrementalencoder")
print("api_getincrementalencoder_is_present OK")
