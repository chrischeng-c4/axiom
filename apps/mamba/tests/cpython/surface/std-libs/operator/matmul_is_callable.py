# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "matmul_is_callable"
# subject = "operator.matmul"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.matmul: matmul_is_callable (surface)."""
import operator

assert callable(operator.matmul)
print("matmul_is_callable OK")
