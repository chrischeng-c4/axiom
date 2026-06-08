# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_filterwarnings_is_present"
# subject = "warnings.filterwarnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.filterwarnings: api_filterwarnings_is_present (surface)."""
import warnings

assert hasattr(warnings, "filterwarnings")
print("api_filterwarnings_is_present OK")
