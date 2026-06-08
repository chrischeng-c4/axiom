# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_resetwarnings_is_present"
# subject = "warnings.resetwarnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.resetwarnings: api_resetwarnings_is_present (surface)."""
import warnings

assert hasattr(warnings, "resetwarnings")
print("api_resetwarnings_is_present OK")
