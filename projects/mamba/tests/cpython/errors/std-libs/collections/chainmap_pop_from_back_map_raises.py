# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "chainmap_pop_from_back_map_raises"
# subject = "collections.ChainMap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: chainmap_pop_from_back_map_raises (errors)."""
import collections

_raised = False
try:
    collections.ChainMap({'a': 1}, {'b': 2}).pop('b')
except KeyError:
    _raised = True
assert _raised, "chainmap_pop_from_back_map_raises: expected KeyError"
print("chainmap_pop_from_back_map_raises OK")
