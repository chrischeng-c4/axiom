# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_waitpid_is_present"
# subject = "os.waitpid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.waitpid: api_waitpid_is_present (surface)."""
import os

assert hasattr(os, "waitpid")
print("api_waitpid_is_present OK")
