# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_catch_warnings_is_present"
# subject = "warnings.catch_warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.catch_warnings: api_catch_warnings_is_present (surface)."""
import warnings

assert hasattr(warnings, "catch_warnings")
print("api_catch_warnings_is_present OK")
