# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ctermid_is_present"
# subject = "os.ctermid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.ctermid: api_ctermid_is_present (surface)."""
import os

assert hasattr(os, "ctermid")
print("api_ctermid_is_present OK")
