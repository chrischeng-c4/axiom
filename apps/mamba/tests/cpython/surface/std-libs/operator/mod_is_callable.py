# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "surface"
# case = "mod_is_callable"
# subject = "operator.mod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.mod: mod_is_callable (surface)."""
import operator

assert callable(operator.mod)
print("mod_is_callable OK")
