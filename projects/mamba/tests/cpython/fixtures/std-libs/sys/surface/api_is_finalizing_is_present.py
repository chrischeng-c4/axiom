# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_is_finalizing_is_present"
# subject = "sys.is_finalizing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.is_finalizing: api_is_finalizing_is_present (surface)."""
import sys

assert hasattr(sys, "is_finalizing")
print("api_is_finalizing_is_present OK")
