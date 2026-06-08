# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sysconf_is_present"
# subject = "os.sysconf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.sysconf: api_sysconf_is_present (surface)."""
import os

assert hasattr(os, "sysconf")
print("api_sysconf_is_present OK")
