# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_replace_errors_is_present"
# subject = "codecs.replace_errors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.replace_errors: api_replace_errors_is_present (surface)."""
import codecs

assert hasattr(codecs, "replace_errors")
print("api_replace_errors_is_present OK")
