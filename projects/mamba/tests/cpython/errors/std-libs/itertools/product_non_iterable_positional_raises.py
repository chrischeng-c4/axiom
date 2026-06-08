# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "product_non_iterable_positional_raises"
# subject = "itertools.product"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.product: product_non_iterable_positional_raises (errors)."""
import itertools

_raised = False
try:
    itertools.product(range(6), None)
except TypeError:
    _raised = True
assert _raised, "product_non_iterable_positional_raises: expected TypeError"
print("product_non_iterable_positional_raises OK")
