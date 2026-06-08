# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_imul_is_present"
# subject = "operator.imul"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.imul: api_imul_is_present (surface)."""
import operator

assert hasattr(operator, "imul")
print("api_imul_is_present OK")
