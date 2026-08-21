# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "surface"
# case = "has_async_iterator"
# subject = "collections.abc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.abc: has_async_iterator (surface)."""
import collections.abc

assert hasattr(collections.abc, "AsyncIterator")
print("has_async_iterator OK")
