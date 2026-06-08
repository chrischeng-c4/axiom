# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_lookup_error_is_present"
# subject = "codecs.lookup_error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.lookup_error: api_lookup_error_is_present (surface)."""
import codecs

assert hasattr(codecs, "lookup_error")
print("api_lookup_error_is_present OK")
