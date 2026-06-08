# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_putenv_is_present"
# subject = "os.putenv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.putenv: api_putenv_is_present (surface)."""
import os

assert hasattr(os, "putenv")
print("api_putenv_is_present OK")
