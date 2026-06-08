# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cprofile"
# dimension = "surface"
# case = "api_run_is_present"
# subject = "cProfile.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""cProfile.run: api_run_is_present (surface)."""
import cProfile

assert hasattr(cProfile, "run")
print("api_run_is_present OK")
