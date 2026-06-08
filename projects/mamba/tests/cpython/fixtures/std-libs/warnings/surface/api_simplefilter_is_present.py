# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "api_simplefilter_is_present"
# subject = "warnings.simplefilter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""warnings.simplefilter: api_simplefilter_is_present (surface)."""
import warnings

assert hasattr(warnings, "simplefilter")
print("api_simplefilter_is_present OK")
