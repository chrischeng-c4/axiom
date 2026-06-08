# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "surface"
# case = "api_product_is_present"
# subject = "itertools.product"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""itertools.product: api_product_is_present (surface)."""
import itertools

assert hasattr(itertools, "product")
print("api_product_is_present OK")
