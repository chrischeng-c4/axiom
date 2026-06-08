# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_rmtree_is_present"
# subject = "shutil.rmtree"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.rmtree: api_rmtree_is_present (surface)."""
import shutil

assert hasattr(shutil, "rmtree")
print("api_rmtree_is_present OK")
