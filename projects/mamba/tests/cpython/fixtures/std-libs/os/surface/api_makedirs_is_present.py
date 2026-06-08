# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_makedirs_is_present"
# subject = "os.makedirs"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.makedirs: api_makedirs_is_present (surface)."""
import os

assert hasattr(os, "makedirs")
print("api_makedirs_is_present OK")
