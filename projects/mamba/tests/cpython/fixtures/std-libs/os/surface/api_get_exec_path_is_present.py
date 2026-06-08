# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_get_exec_path_is_present"
# subject = "os.get_exec_path"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.get_exec_path: api_get_exec_path_is_present (surface)."""
import os

assert hasattr(os, "get_exec_path")
print("api_get_exec_path_is_present OK")
