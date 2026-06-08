# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ex_tempfail_is_present"
# subject = "os.EX_TEMPFAIL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.EX_TEMPFAIL: api_ex_tempfail_is_present (surface)."""
import os

assert hasattr(os, "EX_TEMPFAIL")
print("api_ex_tempfail_is_present OK")
