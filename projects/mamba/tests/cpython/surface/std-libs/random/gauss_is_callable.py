# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "surface"
# case = "gauss_is_callable"
# subject = "random.gauss"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""random.gauss: gauss_is_callable (surface)."""
import random

assert callable(random.gauss)
print("gauss_is_callable OK")
