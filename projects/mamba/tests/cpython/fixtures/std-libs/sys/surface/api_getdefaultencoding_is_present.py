# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getdefaultencoding_is_present"
# subject = "sys.getdefaultencoding"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getdefaultencoding: api_getdefaultencoding_is_present (surface)."""
import sys

assert hasattr(sys, "getdefaultencoding")
print("api_getdefaultencoding_is_present OK")
