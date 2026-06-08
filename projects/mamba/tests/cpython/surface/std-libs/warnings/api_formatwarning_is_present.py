# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_formatwarning_is_present"
# subject = "warnings.formatwarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.formatwarning: api_formatwarning_is_present (surface)."""
import warnings

assert hasattr(warnings, "formatwarning")
print("api_formatwarning_is_present OK")
