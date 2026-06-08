# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_showwarning_is_present"
# subject = "warnings.showwarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.showwarning: api_showwarning_is_present (surface)."""
import warnings

assert hasattr(warnings, "showwarning")
print("api_showwarning_is_present OK")
