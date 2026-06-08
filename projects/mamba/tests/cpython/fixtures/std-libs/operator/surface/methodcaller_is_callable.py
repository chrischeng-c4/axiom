# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "methodcaller_is_callable"
# subject = "operator.methodcaller"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: methodcaller_is_callable (surface)."""
import operator

assert callable(operator.methodcaller)
print("methodcaller_is_callable OK")
