# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "defaultdict_is_callable"
# subject = "collections.defaultdict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict_is_callable (surface)."""
import collections

assert callable(collections.defaultdict)
print("defaultdict_is_callable OK")
