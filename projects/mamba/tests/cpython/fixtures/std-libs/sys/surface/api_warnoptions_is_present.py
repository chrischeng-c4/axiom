# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_warnoptions_is_present"
# subject = "sys.warnoptions"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.warnoptions: api_warnoptions_is_present (surface)."""
import sys

assert hasattr(sys, "warnoptions")
print("api_warnoptions_is_present OK")
