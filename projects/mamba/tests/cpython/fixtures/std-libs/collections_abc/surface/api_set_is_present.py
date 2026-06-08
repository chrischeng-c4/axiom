# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "surface"
# case = "api_set_is_present"
# subject = "collections.abc.Set"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.abc.Set: api_set_is_present (surface)."""
import collections.abc

assert hasattr(collections.abc, "Set")
print("api_set_is_present OK")
