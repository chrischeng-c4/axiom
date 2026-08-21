# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "userdict_is_callable"
# subject = "collections.UserDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserDict: userdict_is_callable (surface)."""
import collections

assert callable(collections.UserDict)
print("userdict_is_callable OK")
