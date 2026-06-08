# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_iterencode_is_present"
# subject = "codecs.iterencode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.iterencode: api_iterencode_is_present (surface)."""
import codecs

assert hasattr(codecs, "iterencode")
print("api_iterencode_is_present OK")
