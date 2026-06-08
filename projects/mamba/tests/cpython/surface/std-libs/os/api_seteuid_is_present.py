# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_seteuid_is_present"
# subject = "os.seteuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.seteuid: api_seteuid_is_present (surface)."""
import os

assert hasattr(os, "seteuid")
print("api_seteuid_is_present OK")
