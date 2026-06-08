# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_hash_is_present"
# subject = "builtins.hash"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.hash: api_hash_is_present (surface)."""
import builtins

assert hasattr(builtins, "hash")
print("api_hash_is_present OK")
