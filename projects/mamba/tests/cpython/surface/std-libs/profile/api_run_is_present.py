# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "surface"
# case = "api_run_is_present"
# subject = "profile.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""profile.run: api_run_is_present (surface)."""
import profile

assert hasattr(profile, "run")
print("api_run_is_present OK")
