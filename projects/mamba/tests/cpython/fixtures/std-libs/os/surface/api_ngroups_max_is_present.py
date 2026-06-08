# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ngroups_max_is_present"
# subject = "os.NGROUPS_MAX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.NGROUPS_MAX: api_ngroups_max_is_present (surface)."""
import os

assert hasattr(os, "NGROUPS_MAX")
print("api_ngroups_max_is_present OK")
