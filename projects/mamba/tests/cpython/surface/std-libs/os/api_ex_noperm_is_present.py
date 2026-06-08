# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ex_noperm_is_present"
# subject = "os.EX_NOPERM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.EX_NOPERM: api_ex_noperm_is_present (surface)."""
import os

assert hasattr(os, "EX_NOPERM")
print("api_ex_noperm_is_present OK")
