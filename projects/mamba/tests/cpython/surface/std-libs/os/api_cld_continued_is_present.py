# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_cld_continued_is_present"
# subject = "os.CLD_CONTINUED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.CLD_CONTINUED: api_cld_continued_is_present (surface)."""
import os

assert hasattr(os, "CLD_CONTINUED")
print("api_cld_continued_is_present OK")
