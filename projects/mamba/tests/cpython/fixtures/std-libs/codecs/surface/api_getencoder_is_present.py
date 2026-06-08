# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_getencoder_is_present"
# subject = "codecs.getencoder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.getencoder: api_getencoder_is_present (surface)."""
import codecs

assert hasattr(codecs, "getencoder")
print("api_getencoder_is_present OK")
