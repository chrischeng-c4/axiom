# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "api_ordered_dict_is_present"
# subject = "collections.OrderedDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.OrderedDict: api_ordered_dict_is_present (surface)."""
import collections

assert hasattr(collections, "OrderedDict")
print("api_ordered_dict_is_present OK")
