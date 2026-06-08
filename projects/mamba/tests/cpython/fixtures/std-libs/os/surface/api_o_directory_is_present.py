# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_o_directory_is_present"
# subject = "os.O_DIRECTORY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.O_DIRECTORY: api_o_directory_is_present (surface)."""
import os

assert hasattr(os, "O_DIRECTORY")
print("api_o_directory_is_present OK")
