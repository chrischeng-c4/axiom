# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cprofile"
# dimension = "surface"
# case = "api_runctx_is_present"
# subject = "cProfile.runctx"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cProfile.runctx: api_runctx_is_present (surface)."""
import cProfile

assert hasattr(cProfile, "runctx")
print("api_runctx_is_present OK")
