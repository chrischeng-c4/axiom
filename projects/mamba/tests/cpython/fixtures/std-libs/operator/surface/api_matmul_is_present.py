# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "api_matmul_is_present"
# subject = "operator.matmul"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""operator.matmul: api_matmul_is_present (surface)."""
import operator

assert hasattr(operator, "matmul")
print("api_matmul_is_present OK")
