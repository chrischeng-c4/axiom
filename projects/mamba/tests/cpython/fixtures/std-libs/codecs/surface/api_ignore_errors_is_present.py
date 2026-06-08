# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_ignore_errors_is_present"
# subject = "codecs.ignore_errors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.ignore_errors: api_ignore_errors_is_present (surface)."""
import codecs

assert hasattr(codecs, "ignore_errors")
print("api_ignore_errors_is_present OK")
