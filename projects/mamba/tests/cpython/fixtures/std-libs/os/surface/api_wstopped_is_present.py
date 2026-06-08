# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wstopped_is_present"
# subject = "os.WSTOPPED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WSTOPPED: api_wstopped_is_present (surface)."""
import os

assert hasattr(os, "WSTOPPED")
print("api_wstopped_is_present OK")
