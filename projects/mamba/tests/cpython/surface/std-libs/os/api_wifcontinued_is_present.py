# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wifcontinued_is_present"
# subject = "os.WIFCONTINUED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WIFCONTINUED: api_wifcontinued_is_present (surface)."""
import os

assert hasattr(os, "WIFCONTINUED")
print("api_wifcontinued_is_present OK")
