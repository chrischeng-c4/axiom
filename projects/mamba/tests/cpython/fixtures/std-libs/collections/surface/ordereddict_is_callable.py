# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "ordereddict_is_callable"
# subject = "collections.OrderedDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: ordereddict_is_callable (surface)."""
import collections

assert callable(collections.OrderedDict)
print("ordereddict_is_callable OK")
