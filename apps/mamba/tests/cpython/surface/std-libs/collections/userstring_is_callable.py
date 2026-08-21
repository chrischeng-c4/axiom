# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "userstring_is_callable"
# subject = "collections.UserString"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.UserString: userstring_is_callable (surface)."""
import collections

assert callable(collections.UserString)
print("userstring_is_callable OK")
