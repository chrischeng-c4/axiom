# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "product_is_callable"
# subject = "itertools.product"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""itertools.product: product_is_callable (surface)."""
import itertools

assert callable(itertools.product)
print("product_is_callable OK")
