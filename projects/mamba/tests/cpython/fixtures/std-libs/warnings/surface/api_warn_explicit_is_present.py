# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_warn_explicit_is_present"
# subject = "warnings.warn_explicit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.warn_explicit: api_warn_explicit_is_present (surface)."""
import warnings

assert hasattr(warnings, "warn_explicit")
print("api_warn_explicit_is_present OK")
