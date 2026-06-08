# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_pow_is_present"
# subject = "operator.pow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.pow: api_pow_is_present (surface)."""
import operator

assert hasattr(operator, "pow")
print("api_pow_is_present OK")
