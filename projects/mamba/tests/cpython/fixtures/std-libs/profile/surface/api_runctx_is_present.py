# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "surface"
# case = "api_runctx_is_present"
# subject = "profile.runctx"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""profile.runctx: api_runctx_is_present (surface)."""
import profile

assert hasattr(profile, "runctx")
print("api_runctx_is_present OK")
