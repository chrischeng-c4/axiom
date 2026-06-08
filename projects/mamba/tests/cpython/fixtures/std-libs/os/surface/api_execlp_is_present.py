# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_execlp_is_present"
# subject = "os.execlp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.execlp: api_execlp_is_present (surface)."""
import os

assert hasattr(os, "execlp")
print("api_execlp_is_present OK")
