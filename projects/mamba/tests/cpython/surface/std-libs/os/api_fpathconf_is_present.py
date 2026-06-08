# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fpathconf_is_present"
# subject = "os.fpathconf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fpathconf: api_fpathconf_is_present (surface)."""
import os

assert hasattr(os, "fpathconf")
print("api_fpathconf_is_present OK")
