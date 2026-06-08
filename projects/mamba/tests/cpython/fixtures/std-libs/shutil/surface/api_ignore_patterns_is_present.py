# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_ignore_patterns_is_present"
# subject = "shutil.ignore_patterns"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.ignore_patterns: api_ignore_patterns_is_present (surface)."""
import shutil

assert hasattr(shutil, "ignore_patterns")
print("api_ignore_patterns_is_present OK")
