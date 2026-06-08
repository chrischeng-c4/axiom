# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cprofile"
# dimension = "surface"
# case = "api_profile_is_present"
# subject = "cProfile.Profile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cProfile.Profile: api_profile_is_present (surface)."""
import cProfile

assert hasattr(cProfile, "Profile")
print("api_profile_is_present OK")
