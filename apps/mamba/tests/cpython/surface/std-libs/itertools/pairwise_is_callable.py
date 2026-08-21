# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "pairwise_is_callable"
# subject = "itertools.pairwise"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.pairwise: pairwise_is_callable (surface)."""
import itertools

assert callable(itertools.pairwise)
print("pairwise_is_callable OK")
