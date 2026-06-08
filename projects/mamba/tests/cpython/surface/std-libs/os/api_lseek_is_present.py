# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_lseek_is_present"
# subject = "os.lseek"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.lseek: api_lseek_is_present (surface)."""
import os

assert hasattr(os, "lseek")
print("api_lseek_is_present OK")
