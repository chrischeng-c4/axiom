# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_geteuid_is_present"
# subject = "os.geteuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.geteuid: api_geteuid_is_present (surface)."""
import os

assert hasattr(os, "geteuid")
print("api_geteuid_is_present OK")
