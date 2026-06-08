# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_seek_cur_is_present"
# subject = "os.SEEK_CUR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.SEEK_CUR: api_seek_cur_is_present (surface)."""
import os

assert hasattr(os, "SEEK_CUR")
print("api_seek_cur_is_present OK")
