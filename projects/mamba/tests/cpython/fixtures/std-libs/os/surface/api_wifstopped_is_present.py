# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wifstopped_is_present"
# subject = "os.WIFSTOPPED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WIFSTOPPED: api_wifstopped_is_present (surface)."""
import os

assert hasattr(os, "WIFSTOPPED")
print("api_wifstopped_is_present OK")
