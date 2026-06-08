# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "matmul_on_ints_typeerror"
# subject = "operator.matmul"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.matmul: matmul_on_ints_typeerror (errors)."""
import operator

_raised = False
try:
    operator.matmul(42, 42)
except TypeError:
    _raised = True
assert _raised, "matmul_on_ints_typeerror: expected TypeError"
print("matmul_on_ints_typeerror OK")
