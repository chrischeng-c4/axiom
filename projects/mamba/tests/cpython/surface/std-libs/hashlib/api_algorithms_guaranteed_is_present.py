# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_algorithms_guaranteed_is_present"
# subject = "hashlib.algorithms_guaranteed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.algorithms_guaranteed: api_algorithms_guaranteed_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "algorithms_guaranteed")
print("api_algorithms_guaranteed_is_present OK")
