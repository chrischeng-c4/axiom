# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_allow_missing_is_present"
# subject = "os.path.ALLOW_MISSING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.ALLOW_MISSING: api_allow_missing_is_present (surface)."""
import os.path

assert hasattr(os.path, "ALLOW_MISSING")
print("api_allow_missing_is_present OK")
