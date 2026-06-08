# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_lchmod_is_present"
# subject = "os.lchmod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.lchmod: api_lchmod_is_present (surface)."""
import os

assert hasattr(os, "lchmod")
print("api_lchmod_is_present OK")
