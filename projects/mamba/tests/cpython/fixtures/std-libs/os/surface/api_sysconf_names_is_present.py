# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sysconf_names_is_present"
# subject = "os.sysconf_names"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.sysconf_names: api_sysconf_names_is_present (surface)."""
import os

assert hasattr(os, "sysconf_names")
print("api_sysconf_names_is_present OK")
