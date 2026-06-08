# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wait4_is_present"
# subject = "os.wait4"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.wait4: api_wait4_is_present (surface)."""
import os

assert hasattr(os, "wait4")
print("api_wait4_is_present OK")
