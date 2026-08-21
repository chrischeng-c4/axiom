# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "groupby_is_callable"
# subject = "itertools.groupby"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.groupby: groupby_is_callable (surface)."""
import itertools

assert callable(itertools.groupby)
print("groupby_is_callable OK")
