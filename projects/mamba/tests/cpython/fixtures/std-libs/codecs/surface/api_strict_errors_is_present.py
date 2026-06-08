# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_strict_errors_is_present"
# subject = "codecs.strict_errors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.strict_errors: api_strict_errors_is_present (surface)."""
import codecs

assert hasattr(codecs, "strict_errors")
print("api_strict_errors_is_present OK")
