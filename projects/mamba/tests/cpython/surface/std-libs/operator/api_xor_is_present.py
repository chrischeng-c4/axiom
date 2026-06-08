# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_xor_is_present"
# subject = "operator.xor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.xor: api_xor_is_present (surface)."""
import operator

assert hasattr(operator, "xor")
print("api_xor_is_present OK")
