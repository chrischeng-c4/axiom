# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_isatty_is_present"
# subject = "os.isatty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.isatty: api_isatty_is_present (surface)."""
import os

assert hasattr(os, "isatty")
print("api_isatty_is_present OK")
