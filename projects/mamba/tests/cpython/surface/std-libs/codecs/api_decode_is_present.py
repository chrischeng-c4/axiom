# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_decode_is_present"
# subject = "codecs.decode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.decode: api_decode_is_present (surface)."""
import codecs

assert hasattr(codecs, "decode")
print("api_decode_is_present OK")
