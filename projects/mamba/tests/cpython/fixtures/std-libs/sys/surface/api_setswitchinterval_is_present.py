# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_setswitchinterval_is_present"
# subject = "sys.setswitchinterval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.setswitchinterval: api_setswitchinterval_is_present (surface)."""
import sys

assert hasattr(sys, "setswitchinterval")
print("api_setswitchinterval_is_present OK")
