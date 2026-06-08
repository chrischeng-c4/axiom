# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setgroups_is_present"
# subject = "os.setgroups"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setgroups: api_setgroups_is_present (surface)."""
import os

assert hasattr(os, "setgroups")
print("api_setgroups_is_present OK")
