# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_execlpe_is_present"
# subject = "os.execlpe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.execlpe: api_execlpe_is_present (surface)."""
import os

assert hasattr(os, "execlpe")
print("api_execlpe_is_present OK")
