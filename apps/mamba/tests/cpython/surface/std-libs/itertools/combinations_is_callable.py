# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "combinations_is_callable"
# subject = "itertools.combinations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.combinations: combinations_is_callable (surface)."""
import itertools

assert callable(itertools.combinations)
print("combinations_is_callable OK")
