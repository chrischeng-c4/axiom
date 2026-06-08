# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_encode_is_present"
# subject = "codecs.encode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.encode: api_encode_is_present (surface)."""
import codecs

assert hasattr(codecs, "encode")
print("api_encode_is_present OK")
