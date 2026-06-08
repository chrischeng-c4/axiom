# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "api_glob_is_present"
# subject = "glob.glob"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""glob.glob: api_glob_is_present (surface)."""
import glob

assert hasattr(glob, "glob")
print("api_glob_is_present OK")
