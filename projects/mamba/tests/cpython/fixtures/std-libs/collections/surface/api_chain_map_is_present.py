# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_chain_map_is_present"
# subject = "collections.ChainMap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.ChainMap: api_chain_map_is_present (surface)."""
import collections

assert hasattr(collections, "ChainMap")
print("api_chain_map_is_present OK")
