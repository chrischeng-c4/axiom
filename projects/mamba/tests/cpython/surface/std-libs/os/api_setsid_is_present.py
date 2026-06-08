# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setsid_is_present"
# subject = "os.setsid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setsid: api_setsid_is_present (surface)."""
import os

assert hasattr(os, "setsid")
print("api_setsid_is_present OK")
