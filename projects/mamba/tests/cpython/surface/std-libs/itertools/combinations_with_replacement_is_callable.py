# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "combinations_with_replacement_is_callable"
# subject = "itertools.combinations_with_replacement"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.combinations_with_replacement: combinations_with_replacement_is_callable (surface)."""
import itertools

assert callable(itertools.combinations_with_replacement)
print("combinations_with_replacement_is_callable OK")
