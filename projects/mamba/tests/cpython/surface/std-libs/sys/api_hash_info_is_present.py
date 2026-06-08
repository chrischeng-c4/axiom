# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_hash_info_is_present"
# subject = "sys.hash_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.hash_info: api_hash_info_is_present (surface)."""
import sys

assert hasattr(sys, "hash_info")
print("api_hash_info_is_present OK")
