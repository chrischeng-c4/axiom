# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "filterfalse_is_callable"
# subject = "itertools.filterfalse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.filterfalse: filterfalse_is_callable (surface)."""
import itertools

assert callable(itertools.filterfalse)
print("filterfalse_is_callable OK")
