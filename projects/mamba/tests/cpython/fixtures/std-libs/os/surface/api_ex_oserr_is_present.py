# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ex_oserr_is_present"
# subject = "os.EX_OSERR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.EX_OSERR: api_ex_oserr_is_present (surface)."""
import os

assert hasattr(os, "EX_OSERR")
print("api_ex_oserr_is_present OK")
