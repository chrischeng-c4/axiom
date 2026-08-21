# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "count_is_callable"
# subject = "itertools.count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.count: count_is_callable (surface)."""
import itertools

assert callable(itertools.count)
print("count_is_callable OK")
