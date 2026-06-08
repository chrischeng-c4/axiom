# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_defaultdict_is_present"
# subject = "collections.defaultdict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.defaultdict: api_defaultdict_is_present (surface)."""
import collections

assert hasattr(collections, "defaultdict")
print("api_defaultdict_is_present OK")
