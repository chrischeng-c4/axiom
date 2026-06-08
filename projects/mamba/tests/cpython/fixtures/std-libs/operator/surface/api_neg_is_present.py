# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_neg_is_present"
# subject = "operator.neg"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.neg: api_neg_is_present (surface)."""
import operator

assert hasattr(operator, "neg")
print("api_neg_is_present OK")
