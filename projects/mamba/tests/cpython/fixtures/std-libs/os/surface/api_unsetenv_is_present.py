# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_unsetenv_is_present"
# subject = "os.unsetenv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.unsetenv: api_unsetenv_is_present (surface)."""
import os

assert hasattr(os, "unsetenv")
print("api_unsetenv_is_present OK")
