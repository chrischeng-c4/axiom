# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "chainmap_is_callable"
# subject = "collections.ChainMap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: chainmap_is_callable (surface)."""
import collections

assert callable(collections.ChainMap)
print("chainmap_is_callable OK")
