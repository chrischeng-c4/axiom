# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getfilesystemencoding_is_present"
# subject = "sys.getfilesystemencoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getfilesystemencoding: api_getfilesystemencoding_is_present (surface)."""
import sys

assert hasattr(sys, "getfilesystemencoding")
print("api_getfilesystemencoding_is_present OK")
