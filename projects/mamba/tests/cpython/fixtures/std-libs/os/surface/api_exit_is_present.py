# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_exit_is_present"
# subject = "os._exit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os._exit: api_exit_is_present (surface)."""
import os

assert hasattr(os, "_exit")
print("api_exit_is_present OK")
