# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_execvpe_is_present"
# subject = "os.execvpe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.execvpe: api_execvpe_is_present (surface)."""
import os

assert hasattr(os, "execvpe")
print("api_execvpe_is_present OK")
