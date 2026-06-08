# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_terminal_size_is_present"
# subject = "os.terminal_size"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.terminal_size: api_terminal_size_is_present (surface)."""
import os

assert hasattr(os, "terminal_size")
print("api_terminal_size_is_present OK")
