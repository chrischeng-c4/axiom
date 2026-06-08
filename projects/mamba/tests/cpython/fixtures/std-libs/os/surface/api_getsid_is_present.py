# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getsid_is_present"
# subject = "os.getsid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getsid: api_getsid_is_present (surface)."""
import os

assert hasattr(os, "getsid")
print("api_getsid_is_present OK")
