# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "surface"
# case = "api_profile_is_present"
# subject = "profile.Profile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""profile.Profile: api_profile_is_present (surface)."""
import profile

assert hasattr(profile, "Profile")
print("api_profile_is_present OK")
