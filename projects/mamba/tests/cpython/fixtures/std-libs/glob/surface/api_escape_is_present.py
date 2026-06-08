# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "surface"
# case = "api_escape_is_present"
# subject = "glob.escape"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""glob.escape: api_escape_is_present (surface)."""
import glob

assert hasattr(glob, "escape")
print("api_escape_is_present OK")
