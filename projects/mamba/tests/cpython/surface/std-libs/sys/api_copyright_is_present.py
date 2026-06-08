# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_copyright_is_present"
# subject = "sys.copyright"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.copyright: api_copyright_is_present (surface)."""
import sys

assert hasattr(sys, "copyright")
print("api_copyright_is_present OK")
