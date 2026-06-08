# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_get_terminal_size_is_present"
# subject = "shutil.get_terminal_size"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.get_terminal_size: api_get_terminal_size_is_present (surface)."""
import shutil

assert hasattr(shutil, "get_terminal_size")
print("api_get_terminal_size_is_present OK")
