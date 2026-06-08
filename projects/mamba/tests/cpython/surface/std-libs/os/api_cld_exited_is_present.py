# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_cld_exited_is_present"
# subject = "os.CLD_EXITED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.CLD_EXITED: api_cld_exited_is_present (surface)."""
import os

assert hasattr(os, "CLD_EXITED")
print("api_cld_exited_is_present OK")
