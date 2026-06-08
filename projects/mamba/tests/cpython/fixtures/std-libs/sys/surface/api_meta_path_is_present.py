# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_meta_path_is_present"
# subject = "sys.meta_path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.meta_path: api_meta_path_is_present (surface)."""
import sys

assert hasattr(sys, "meta_path")
print("api_meta_path_is_present OK")
