# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "api_namereplace_errors_is_present"
# subject = "codecs.namereplace_errors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""codecs.namereplace_errors: api_namereplace_errors_is_present (surface)."""
import codecs

assert hasattr(codecs, "namereplace_errors")
print("api_namereplace_errors_is_present OK")
