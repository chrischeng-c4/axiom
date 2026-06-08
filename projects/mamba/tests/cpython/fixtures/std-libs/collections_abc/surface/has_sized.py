# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "surface"
# case = "has_sized"
# subject = "collections.abc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc: has_sized (surface)."""
import collections.abc

assert hasattr(collections.abc, "Sized")
print("has_sized OK")
