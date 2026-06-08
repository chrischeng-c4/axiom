# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "surface"
# case = "isrecursive_is_callable"
# subject = "pprint.isrecursive"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.isrecursive: isrecursive_is_callable (surface)."""
import pprint

assert callable(pprint.isrecursive)
print("isrecursive_is_callable OK")
