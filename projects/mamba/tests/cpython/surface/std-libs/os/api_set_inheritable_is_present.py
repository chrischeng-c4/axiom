# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_set_inheritable_is_present"
# subject = "os.set_inheritable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.set_inheritable: api_set_inheritable_is_present (surface)."""
import os

assert hasattr(os, "set_inheritable")
print("api_set_inheritable_is_present OK")
