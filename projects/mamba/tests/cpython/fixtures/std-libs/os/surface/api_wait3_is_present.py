# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wait3_is_present"
# subject = "os.wait3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.wait3: api_wait3_is_present (surface)."""
import os

assert hasattr(os, "wait3")
print("api_wait3_is_present OK")
