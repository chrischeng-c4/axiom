# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_version_info_is_present"
# subject = "sys.version_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.version_info: api_version_info_is_present (surface)."""
import sys

assert hasattr(sys, "version_info")
print("api_version_info_is_present OK")
