# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_platform_is_present"
# subject = "sys.platform"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.platform: api_platform_is_present (surface)."""
import sys

assert hasattr(sys, "platform")
print("api_platform_is_present OK")
