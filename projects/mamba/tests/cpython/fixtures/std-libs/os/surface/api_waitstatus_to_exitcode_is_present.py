# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_waitstatus_to_exitcode_is_present"
# subject = "os.waitstatus_to_exitcode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.waitstatus_to_exitcode: api_waitstatus_to_exitcode_is_present (surface)."""
import os

assert hasattr(os, "waitstatus_to_exitcode")
print("api_waitstatus_to_exitcode_is_present OK")
