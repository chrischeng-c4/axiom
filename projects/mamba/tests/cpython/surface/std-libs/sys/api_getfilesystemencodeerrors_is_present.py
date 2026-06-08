# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getfilesystemencodeerrors_is_present"
# subject = "sys.getfilesystemencodeerrors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getfilesystemencodeerrors: api_getfilesystemencodeerrors_is_present (surface)."""
import sys

assert hasattr(sys, "getfilesystemencodeerrors")
print("api_getfilesystemencodeerrors_is_present OK")
