# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_get_terminal_size_is_present"
# subject = "os.get_terminal_size"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.get_terminal_size: api_get_terminal_size_is_present (surface)."""
import os

assert hasattr(os, "get_terminal_size")
print("api_get_terminal_size_is_present OK")
