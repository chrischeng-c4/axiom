# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wexitstatus_is_present"
# subject = "os.WEXITSTATUS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WEXITSTATUS: api_wexitstatus_is_present (surface)."""
import os

assert hasattr(os, "WEXITSTATUS")
print("api_wexitstatus_is_present OK")
