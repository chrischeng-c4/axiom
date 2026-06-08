# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_warn_is_present"
# subject = "warnings.warn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.warn: api_warn_is_present (surface)."""
import warnings

assert hasattr(warnings, "warn")
print("api_warn_is_present OK")
