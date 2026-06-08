# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_namedtuple_is_present"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.namedtuple: api_namedtuple_is_present (surface)."""
import collections

assert hasattr(collections, "namedtuple")
print("api_namedtuple_is_present OK")
