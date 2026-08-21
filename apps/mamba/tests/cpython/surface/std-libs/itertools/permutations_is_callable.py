# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "permutations_is_callable"
# subject = "itertools.permutations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.permutations: permutations_is_callable (surface)."""
import itertools

assert callable(itertools.permutations)
print("permutations_is_callable OK")
