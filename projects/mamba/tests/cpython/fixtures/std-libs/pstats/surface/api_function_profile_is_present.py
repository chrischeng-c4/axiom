# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "surface"
# case = "api_function_profile_is_present"
# subject = "pstats.FunctionProfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pstats.FunctionProfile: api_function_profile_is_present (surface)."""
import pstats

assert hasattr(pstats, "FunctionProfile")
print("api_function_profile_is_present OK")
