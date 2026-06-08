# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_chmod_is_present"
# subject = "os.chmod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.chmod: api_chmod_is_present (surface)."""
import os

assert hasattr(os, "chmod")
print("api_chmod_is_present OK")
