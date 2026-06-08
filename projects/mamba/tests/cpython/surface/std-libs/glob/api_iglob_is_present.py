# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "api_iglob_is_present"
# subject = "glob.iglob"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""glob.iglob: api_iglob_is_present (surface)."""
import glob

assert hasattr(glob, "iglob")
print("api_iglob_is_present OK")
