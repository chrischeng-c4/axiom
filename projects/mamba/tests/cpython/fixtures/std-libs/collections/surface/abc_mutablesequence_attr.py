# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "surface"
# case = "abc_mutablesequence_attr"
# subject = "collections.abc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc: abc_mutablesequence_attr (surface)."""
import collections.abc

assert hasattr(collections.abc, "MutableSequence")
print("abc_mutablesequence_attr OK")
