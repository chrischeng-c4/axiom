# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "chain_from_iterable_is_callable"
# subject = "itertools.chain.from_iterable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.chain.from_iterable: chain_from_iterable_is_callable (surface)."""
import itertools

assert callable(itertools.chain.from_iterable)
print("chain_from_iterable_is_callable OK")
