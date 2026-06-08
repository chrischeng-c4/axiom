# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_algorithms_available_is_present"
# subject = "hashlib.algorithms_available"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.algorithms_available: api_algorithms_available_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "algorithms_available")
print("api_algorithms_available_is_present OK")
