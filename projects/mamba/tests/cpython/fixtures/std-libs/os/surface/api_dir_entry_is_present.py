# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_dir_entry_is_present"
# subject = "os.DirEntry"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.DirEntry: api_dir_entry_is_present (surface)."""
import os

assert hasattr(os, "DirEntry")
print("api_dir_entry_is_present OK")
