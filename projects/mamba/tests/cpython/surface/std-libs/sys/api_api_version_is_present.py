# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_api_version_is_present"
# subject = "sys.api_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.api_version: api_api_version_is_present (surface)."""
import sys

assert hasattr(sys, "api_version")
print("api_api_version_is_present OK")
