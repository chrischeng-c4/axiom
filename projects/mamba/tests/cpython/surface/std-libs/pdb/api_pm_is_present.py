# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "surface"
# case = "api_pm_is_present"
# subject = "pdb.pm"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pdb.pm: api_pm_is_present (surface)."""
import pdb

assert hasattr(pdb, "pm")
print("api_pm_is_present OK")
