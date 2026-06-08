# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_lockf_is_present"
# subject = "os.lockf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.lockf: api_lockf_is_present (surface)."""
import os

assert hasattr(os, "lockf")
print("api_lockf_is_present OK")
