# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_copytree_is_present"
# subject = "shutil.copytree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.copytree: api_copytree_is_present (surface)."""
import shutil

assert hasattr(shutil, "copytree")
print("api_copytree_is_present OK")
