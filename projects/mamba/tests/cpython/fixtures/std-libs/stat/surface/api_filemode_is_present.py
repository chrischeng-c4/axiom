# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_filemode_is_present"
# subject = "stat.filemode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.filemode: api_filemode_is_present (surface)."""
import stat

assert hasattr(stat, "filemode")
print("api_filemode_is_present OK")
