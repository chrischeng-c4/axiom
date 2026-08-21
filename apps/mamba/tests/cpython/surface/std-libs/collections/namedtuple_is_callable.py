# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "namedtuple_is_callable"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: namedtuple_is_callable (surface)."""
import collections

assert callable(collections.namedtuple)
print("namedtuple_is_callable OK")
